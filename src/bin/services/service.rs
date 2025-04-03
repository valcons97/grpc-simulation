use crate::proto::proto::simulation_server::Simulation;
use crate::proto::proto::{HelloRequest, HelloResponse};
use crypto::{decrypt_message, encrypt_message};
use futures::StreamExt;
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};

mod crypto;
mod proto;

type ResponseStream = Pin<Box<dyn futures::Stream<Item = Result<HelloResponse, Status>> + Send>>;

#[derive(Debug, Default)]
pub struct SimulationService;

impl SimulationService {
    const AES_KEY: [u8; 16] = [0x00; 16]; // Please change your encryption key later or use .env
}

#[tonic::async_trait]
impl Simulation for SimulationService {
    async fn unary_rpc(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let message = request.into_inner().message;

        let decrypted_message = decrypt_message(&Self::AES_KEY, &message);

        let reply = HelloResponse {
            message: format!("You got, {} message!", decrypted_message),
        };

        let encrypted_reply = encrypt_message(&Self::AES_KEY, &reply.message);

        Ok(Response::new(HelloResponse {
            message: encrypted_reply,
        }))
    }

    type ServerStreamingRPCStream = ResponseStream;

    async fn server_streaming_rpc(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::ServerStreamingRPCStream>, Status> {
        let message = request.into_inner().message;

        let decrypted_message = decrypt_message(&Self::AES_KEY, &message);

        let stream_message = async_stream::stream! {
            for i in 1..=3 {
                let response_message = format!("Hello, {}! ({} times)", decrypted_message, i);

                let encrypted_response = encrypt_message(&Self::AES_KEY, &response_message);

                yield Ok(HelloResponse {
                    message: encrypted_response,
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
        let mut decrypted_messages = Vec::new();
        let mut request_stream = request.into_inner();

        while let Some(req) = request_stream.next().await {
            match req {
                Ok(hello_request) => {
                    let decrypted_message = decrypt_message(&Self::AES_KEY, &hello_request.message);
                    decrypted_messages.push(decrypted_message);
                }
                Err(e) => return Err(Status::invalid_argument(e.to_string())),
            }
        }

        let reply_message = format!("Hello, {}!", decrypted_messages.join(", "));
        let encrypted_reply = encrypt_message(&Self::AES_KEY, &reply_message);

        let reply = HelloResponse {
            message: encrypted_reply,
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
