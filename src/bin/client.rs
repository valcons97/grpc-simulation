use crate::proto::{HelloRequest, simulation_client::SimulationClient};
use dotenv::dotenv;
use futures::stream;
use std::env;
use std::time::Duration;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

pub mod proto {
    tonic::include_proto!("grpc.simulation");
}

fn hello_requests_iter() -> impl Stream<Item = HelloRequest> {
    tokio_stream::iter(1..usize::MAX).map(|i| HelloRequest {
        message: format!("msg {:02}", i),
    })
}

async fn single_rpc(client: &mut SimulationClient<Channel>) {
    let request = HelloRequest {
        message: "Hello from client!".to_string(),
    };

    match client.unary_rpc(request).await {
        Ok(response) => {
            println!("Response: {:?}", response.get_ref().message);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}

async fn streaming_client_rpc(client: &mut SimulationClient<Channel>, num: usize) {
    let requests = stream::iter((1..=num).map(|i| HelloRequest {
        message: format!("Message {}", i),
    }));

    let response = client.client_streaming_rpc(requests).await.unwrap();

    let reply = response.into_inner();
    println!("Received: {}", reply.message);
}

async fn streaming_server_rpc(client: &mut SimulationClient<Channel>, num: usize) {
    let stream = client
        .server_streaming_rpc(HelloRequest {
            message: "from client".into(),
        })
        .await
        .unwrap()
        .into_inner();

    let mut stream = stream.take(num);
    while let Some(item) = stream.next().await {
        println!("\treceived: {}", item.unwrap().message);
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
        println!("\treceived message: `{}`", received.message);
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

    println!("Unary rpc:");
    single_rpc(&mut client).await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("Streaming client:");
    streaming_client_rpc(&mut client, 5).await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("Streaming server:");
    streaming_server_rpc(&mut client, 5).await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("\r\nBidirectional stream echo:");
    bidirectional_streaming_rpc(&mut client, 17).await;

    Ok(())
}
