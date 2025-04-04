use crate::client_service::*;
use crate::proto::proto::simulation_client::SimulationClient;
use dotenv::dotenv;
use std::env;
use std::time::Duration;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

#[path = "services/client_service.rs"]
mod client_service;
#[path = "services/crypto.rs"]
mod crypto;
#[path = "services/proto.rs"]
mod proto;

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
