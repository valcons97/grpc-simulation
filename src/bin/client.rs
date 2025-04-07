use crate::client_service::*;
use crate::global::Global;
use crate::proto::proto::simulation_client::SimulationClient;
use http::Uri;
use std::time::Duration;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

#[path = "services/client_service.rs"]
mod client_service;
#[path = "services/crypto.rs"]
mod crypto;
#[path = "services/static_variables.rs"]
mod global;
#[path = "services/proto.rs"]
mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let config = Global::new();

    let server_root_ca_cert = std::fs::read_to_string(config.server_root_file_path)?;
    let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);
    let client_cert = std::fs::read_to_string(config.cert_file_path)?;
    let client_key = std::fs::read_to_string(config.key_file_path)?;
    let client_identity = Identity::from_pem(client_cert, client_key);

    let tls = ClientTlsConfig::new()
        .domain_name(config.server_domain_name)
        .ca_certificate(server_root_ca_cert)
        .identity(client_identity);

    let server_url: Uri = config.server_url.parse().expect("Invalid server URL");

    let channel = Channel::builder(server_url)
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
