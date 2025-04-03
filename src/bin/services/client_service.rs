use crate::crypto::{decrypt_message, encrypt_message};
use crate::proto::proto::{HelloRequest, simulation_client::SimulationClient};
use futures::stream;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Channel;

fn hello_requests_iter(aes_key: &[u8; 16]) -> impl Stream<Item = HelloRequest> {
    tokio_stream::iter(1..usize::MAX).map(move |i| {
        let message = format!("msg {:02}", i);
        let encrypted_message = encrypt_message(aes_key, &message);
        HelloRequest {
            message: encrypted_message,
        }
    })
}

const AES_KEY: [u8; 16] = [0x00; 16]; // Please change your encryption key later or use .env

pub async fn single_rpc(client: &mut SimulationClient<Channel>) {
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

pub async fn streaming_client_rpc(client: &mut SimulationClient<Channel>, num: usize) {
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

pub async fn streaming_server_rpc(client: &mut SimulationClient<Channel>, num: usize) {
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

pub async fn bidirectional_streaming_rpc(client: &mut SimulationClient<Channel>, num: usize) {
    let in_stream = hello_requests_iter(&AES_KEY).take(num);

    let response = client
        .bi_directional_streaming_rpc(in_stream)
        .await
        .unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();

        let decrypted_message = decrypt_message(&AES_KEY, &received.message);
        println!("\tResponse: `{}`", decrypted_message);
    }
}
