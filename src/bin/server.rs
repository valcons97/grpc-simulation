use crate::global::Global;
use crate::proto::proto::simulation_server::SimulationServer;
use crate::service::SimulationService;
use regex::Regex;
use std::net::SocketAddr;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

#[path = "services/static_variables.rs"]
mod global;
#[path = "services/proto.rs"]
mod proto;
#[path = "services/service.rs"]
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Global::new();

    let cert = std::fs::read_to_string(config.cert_file_path)?;
    let key = std::fs::read_to_string(config.key_file_path)?;
    let identity = Identity::from_pem(cert, key);

    let client_ca_cert = std::fs::read_to_string(config.server_client_path)?;
    let client_ca_cert = Certificate::from_pem(client_ca_cert);

    let addr: SocketAddr = Regex::new(r"^https://")?
        .replace(&config.server_url, "")
        .parse()
        .expect("Failed to parse server URL");

    let service = SimulationService::default();

    let tls = ServerTlsConfig::new()
        .identity(identity)
        .client_ca_root(client_ca_cert);

    let server_future = Server::builder()
        .tls_config(tls)?
        .add_service(SimulationServer::new(service))
        .serve(addr);

    println!("Server is running on {}", addr);

    server_future.await?;

    Ok(())
}
