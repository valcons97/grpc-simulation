use std::{error::Error, io::ErrorKind, net::ToSocketAddrs, pin::Pin, time::Duration};

use crate::proto::simulation_server::{Simulation, SimulationServer};
use crate::proto::{HelloRequest, HelloResponse};
use futures::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

type ResponseStream = Pin<Box<dyn futures::Stream<Item = Result<HelloResponse, Status>> + Send>>;

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

    // This defines the type for the server streaming RPC's stream.
    type ServerStreamingRPCStream = ResponseStream;

    // Server Streaming RPC: Handles a single request and returns a stream of responses.
    async fn server_streaming_rpc(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::ServerStreamingRPCStream>, Status> {
        let message = request.into_inner().message;

        let stream_message = async_stream::stream! {
            for i in 1..=3 {
                yield Ok(HelloResponse { // Wrap the HelloResponse in Result
                    message: format!("Hello, {}! ({} times)", message, i),
                });
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        };

        let boxed_stream = Box::pin(stream_message);

        Ok(Response::new(boxed_stream))
    }
}
