use crate::service::SimulationService;
use grpc_simulation::proto::{self, simulation_server::SimulationServer};
use std::fs::File;
use std::io::Read;
use tonic::{
    Request, Response, Status,
    transport::{
        Identity, Server, ServerTlsConfig,
        server::{TcpConnectInfo, TlsConnectInfo},
    },
};

mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load server certificates and key for mTLS
    let cert_file = &mut File::open("server.crt")?;
    let mut cert_buf = Vec::new();
    cert_file.read_to_end(&mut cert_buf)?;

    let key_file = &mut File::open("server.key")?;
    let mut key_buf = Vec::new();
    key_file.read_to_end(&mut key_buf)?;

    let identity = Identity::from_pem(cert_buf, key_buf);

    let tls_config = ServerTlsConfig::new().identity(identity);

    // Setup server address
    let addr = "[::1]:50051".parse()?;
    let service = SimulationService::default();

    // Start the server with mTLS
    Server::builder()
        .tls_config(tls_config)?
        .add_service(SimulationServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
