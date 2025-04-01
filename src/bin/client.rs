use crate::proto::{HelloRequest, simulation_client::SimulationClient};
use std::fs::File;
use std::io::Read;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

pub mod proto {
    tonic::include_proto!("grpc.simulation");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ca_cert_file = File::open("ca.crt")?;
    let mut ca_cert_buf = Vec::new();
    ca_cert_file.read_to_end(&mut ca_cert_buf)?;

    let ca_certificate = Certificate::from_pem(ca_cert_buf);

    let tls = ClientTlsConfig::new().ca_certificate(ca_certificate);

    let channel = Channel::from_static("https://[::1]:50051")
        .tls_config(tls)?
        .connect()
        .await?;

    let mut client = SimulationClient::new(channel);

    // Create a request and send it
    let request = HelloRequest {
        message: "Hello from client!".to_string(),
    };

    let response = client.unary_rpc(request).await?;
    println!("Response: {:?}", response.into_inner().message);

    Ok(())
}
