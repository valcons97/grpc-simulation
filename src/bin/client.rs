use crate::crypto::{decrypt_message, encrypt_message};
use crate::proto::proto::{HelloRequest, simulation_client::SimulationClient};
use dotenv::dotenv;
use futures::stream;
use std::env;
use std::time::Duration;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

#[path = "services/crypto.rs"]
mod crypto;
#[path = "services/proto.rs"]
mod proto;

fn hello_requests_iter() -> impl Stream<Item = HelloRequest> {
    tokio_stream::iter(1..usize::MAX).map(|i| HelloRequest {
        message: format!("msg {:02}", i),
    })
}

const AES_KEY: [u8; 16] = [0x00; 16]; // Please change your encryption key later or use .env

async fn single_rpc(client: &mut SimulationClient<Channel>) {
    let original_message = "Hello from client!";

    let encrypted_message = encrypt_message(&AES_KEY, original_message);

    let request = HelloRequest {
        message: encrypted_message,
    };

    match client.unary_rpc(request).await {
        Ok(response) => {
            let decrypted_response = decrypt_message(&AES_KEY, response.get_ref().message.as_str());
            println!("\tResponse: {:?}", decrypted_response);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}

async fn streaming_client_rpc(client: &mut SimulationClient<Channel>, num: usize) {
    let requests = stream::iter((1..=num).map(|i| {
        let message = format!("Message {}", i);
        let encrypted_message = encrypt_message(&AES_KEY, &message);
        HelloRequest {
            message: encrypted_message,
        }
    }));

    let response = client.client_streaming_rpc(requests).await.unwrap();

    let reply = response.into_inner();

    let decrypted_reply = decrypt_message(&AES_KEY, &reply.message);

    println!("\tResponse: {}", decrypted_reply);
}

async fn streaming_server_rpc(client: &mut SimulationClient<Channel>, num: usize) {
    let original_message = "from client".to_string();
    let encrypted_request = encrypt_message(&AES_KEY, &original_message);

    let stream = client
        .server_streaming_rpc(HelloRequest {
            message: encrypted_request,
        })
        .await
        .unwrap()
        .into_inner();

    let mut stream = stream.take(num);
    while let Some(item) = stream.next().await {
        match item {
            Ok(encrypted_response) => {
                let decrypted_message = decrypt_message(&AES_KEY, &encrypted_response.message);
                println!("\tResponse: {}", decrypted_message);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        }
    }
}

async fn bidirectional_streaming_rpc(client: &mut SimulationClient<Channel>, num: usize) {
    let in_stream = hello_requests_iter().take(num);

    let response = client
        .bi_directional_streaming_rpc(in_stream)
        .await
        .unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();
        println!("\tResponse: `{}`", received.message);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let cert_file_path = env::var("CLIENT_CERT_PATH").expect("CLIENT_CERT_PATH not set");
    let server_root_file_path = env::var("SERVER_ROOT_PATH").expect("SERVER_ROOT_PATH not set");
    let key_file_path = env::var("CLIENT_KEY_PATH").expect("CLIENT_KEY_PATH not set");

    let server_root_ca_cert = std::fs::read_to_string(server_root_file_path)?;
    let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);
    let client_cert = std::fs::read_to_string(cert_file_path)?;
    let client_key = std::fs::read_to_string(key_file_path)?;
    let client_identity = Identity::from_pem(client_cert, client_key);

    let tls = ClientTlsConfig::new()
        .domain_name("localhost")
        .ca_certificate(server_root_ca_cert)
        .identity(client_identity);

    let channel = Channel::from_static("https://[::1]:50051")
        .tls_config(tls)?
        .connect()
        .await?;

    let mut client = SimulationClient::new(channel);

    println!("\r\nUnary rpc:");
    single_rpc(&mut client).await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("\r\nStreaming client:");
    streaming_client_rpc(&mut client, 5).await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("\r\nStreaming server:");
    streaming_server_rpc(&mut client, 5).await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("\r\nBidirectional stream rpc:");
    bidirectional_streaming_rpc(&mut client, 17).await;

    Ok(())
}
