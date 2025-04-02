use crate::proto::simulation_server::{Simulation, SimulationServer};
use crate::proto::{HelloRequest, HelloResponse};
use dotenv::dotenv;
use futures::StreamExt;
use std::env;
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status, Streaming};

type ResponseStream = Pin<Box<dyn futures::Stream<Item = Result<HelloResponse, Status>> + Send>>;

pub mod proto {
    tonic::include_proto!("grpc.simulation");
}

#[derive(Debug, Default)]
pub struct SimulationService;

#[tonic::async_trait]
impl Simulation for SimulationService {
    async fn unary_rpc(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let message = request.into_inner().message;
        let reply = HelloResponse {
            message: format!("You got, {} message!", message),
        };
        Ok(Response::new(reply))
    }

    type ServerStreamingRPCStream = ResponseStream;

    async fn server_streaming_rpc(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::ServerStreamingRPCStream>, Status> {
        let message = request.into_inner().message;

        let stream_message = async_stream::stream! {
            for i in 1..=3 {
                yield Ok(HelloResponse {
                    message: format!("Hello, {}! ({} times)", message, i),
                });
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        };

        let boxed_stream = Box::pin(stream_message);

        Ok(Response::new(boxed_stream))
    }

    async fn client_streaming_rpc(
        &self,
        request: Request<Streaming<HelloRequest>>,
    ) -> Result<Response<HelloResponse>, Status> {
        let mut message = Vec::new();
        let mut request_stream = request.into_inner();

        while let Some(req) = request_stream.next().await {
            match req {
                Ok(hello_request) => message.push(hello_request.message),
                Err(e) => return Err(Status::invalid_argument(e.to_string())),
            }
        }

        let reply = HelloResponse {
            message: format!("Hello, {}!", message.join(", ")),
        };

        Ok(Response::new(reply))
    }

    type BiDirectionalStreamingRpcStream = ResponseStream;

    async fn bi_directional_streaming_rpc(
        &self,
        request: Request<Streaming<HelloRequest>>,
    ) -> Result<Response<Self::BiDirectionalStreamingRpcStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let mut request_stream = request.into_inner();

        tokio::spawn(async move {
            while let Some(req) = request_stream.next().await {
                match req {
                    Ok(hello_request) => {
                        let reply = HelloResponse {
                            message: format!("Hello, {}!", hello_request.message),
                        };
                        if tx.send(Ok(reply)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        Ok(Response::new(
            Box::pin(stream) as Self::BiDirectionalStreamingRpcStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let cert_file_path = env::var("SERVER_CERT_PATH").expect("SERVER_CERT_PATH not set");
    let key_file_path = env::var("SERVER_KEY_PATH").expect("SERVER_KEY_PATH not set");
    let client_file_path = env::var("SERVER_CLIENT_PATH").expect("SERVER_CLIENT_PATH not set");

    let cert = std::fs::read_to_string(cert_file_path)?;
    let key = std::fs::read_to_string(key_file_path)?;
    let identity = Identity::from_pem(cert, key);

    let client_ca_cert = std::fs::read_to_string(client_file_path)?;
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
