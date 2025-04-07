use crate::proto::proto::simulation_server::Simulation;
use crate::proto::proto::{HelloRequest, HelloResponse};
use crypto::{decrypt_message, encrypt_message};
use futures::StreamExt;
use static_variables::Global;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};

mod crypto;
mod proto;
mod static_variables;

type ResponseStream = Pin<Box<dyn futures::Stream<Item = Result<HelloResponse, Status>> + Send>>;

#[derive(Debug, Default)]
pub struct SimulationService {
    aes_key: [u8; 16],
}

impl SimulationService {
    pub fn new(aes_key: [u8; 16]) -> Self {
        Self { aes_key }
    }

    // Function to create a service with the default AES key from config
    pub fn from_config() -> Self {
        let config = Global::new();
        let aes_key = config.aes_key;
        Self::new(aes_key)
    }
}

#[tonic::async_trait]
impl Simulation for SimulationService {
    async fn unary_rpc(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let message = request.into_inner().message;

        let decrypted_message = decrypt_message(&self.aes_key, &message);

        let reply = HelloResponse {
            message: format!("You got, {} message!", decrypted_message),
        };

        let encrypted_reply = encrypt_message(&self.aes_key, &reply.message);

        Ok(Response::new(HelloResponse {
            message: encrypted_reply,
        }))
    }

    type ServerStreamingRPCStream = ResponseStream;

    async fn server_streaming_rpc(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::ServerStreamingRPCStream>, Status> {
        let aes_key = Arc::new(self.aes_key.clone());

        let message = request.into_inner().message;

        let decrypted_message = decrypt_message(&self.aes_key, &message);

        let stream_message = async_stream::stream! {
            for i in 1..=3 {
                let response_message = format!("Hello, {}! ({} times)", decrypted_message, i);
                let encrypted_response = encrypt_message(&aes_key, &response_message);

                yield Ok(HelloResponse {
                    message: encrypted_response,
                });
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        };

        let boxed_stream: ResponseStream = Box::pin(stream_message);

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
                    let decrypted_message = decrypt_message(&self.aes_key, &hello_request.message);
                    decrypted_messages.push(decrypted_message);
                }
                Err(e) => return Err(Status::invalid_argument(e.to_string())),
            }
        }

        let reply_message = format!("Hello, {}!", decrypted_messages.join(", "));
        let encrypted_reply = encrypt_message(&self.aes_key, &reply_message);

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
        let aes_key = Arc::new(self.aes_key.clone());

        tokio::spawn(async move {
            while let Some(req) = request_stream.next().await {
                match req {
                    Ok(hello_request) => {
                        let decrypted_message = decrypt_message(&aes_key, &hello_request.message);

                        let reply = HelloResponse {
                            message: format!("Hello, {}!", decrypted_message),
                        };

                        let encrypted_reply = encrypt_message(&aes_key, &reply.message);

                        if tx
                            .send(Ok(HelloResponse {
                                message: encrypted_reply,
                            }))
                            .await
                            .is_err()
                        {
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
