use crate::global::Global;
use crate::proto::proto::simulation_server::SimulationServer;
use crate::service::SimulationService;
use dotenv::dotenv;
use std::env;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

#[path = "services/static_variables.rs"]
mod global;
#[path = "services/proto.rs"]
mod proto;
#[path = "services/service.rs"]
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let config = Global::new();

    let cert = std::fs::read_to_string(config.cert_file_path)?;
    let key = std::fs::read_to_string(config.key_file_path)?;
    let identity = Identity::from_pem(cert, key);

    let client_ca_cert = std::fs::read_to_string(config.server_client_path)?;
    let client_ca_cert = Certificate::from_pem(client_ca_cert);

    let addr = "[::1]:50051".parse()?;
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
