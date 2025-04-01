use crate::proto::simulation_server::{Simulation, SimulationServer};
use crate::proto::{HelloRequest, HelloResponse};
use futures::StreamExt;
use std::fs::File;
use std::io::Read;
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status, Streaming};

type ResponseStream = Pin<Box<dyn futures::Stream<Item = Result<HelloResponse, Status>> + Send>>;

pub mod proto {
    tonic::include_proto!("grpc.simulation");
}

#[derive(Debug, Default)]
pub struct SimulationService;

#[tonic::async_trait]
impl Simulation for SimulationService {
    // Unary RPC: Handles a single request and returns a single response.
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

    // Server Streaming RPC: Handles a single request and returns a stream of responses.
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

    // Client Streaming RPC: Handles a stream of requests and returns a single response.
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
        let mut request_stream = request.into_inner(); // Extract the stream from the request

        // Spawn a task to process the stream and send back responses
        tokio::spawn(async move {
            while let Some(req) = request_stream.next().await {
                match req {
                    Ok(hello_request) => {
                        let reply = HelloResponse {
                            message: format!("Hello, {}!", hello_request.message),
                        };
                        if tx.send(Ok(reply)).await.is_err() {
                            break; // Channel closed, stop processing.
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // We need to wrap the receiver (rx) to match the expected stream type:
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        // The return type must be a Pin<Box<dyn Stream<Item = Result<HelloResponse, Status>> + Send>>
        Ok(Response::new(
            Box::pin(stream) as Self::BiDirectionalStreamingRpcStream
        ))
    }
}

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
