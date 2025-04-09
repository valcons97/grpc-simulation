#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::Request;

    fn setup_service() -> SimulationService {
        SimulationService::new([0u8; 16]) // use a fixed AES key
    }

    #[tokio::test]
    async fn test_unary_rpc() {
        let service = setup_service();
        let message = encrypt_message(&service.aes_key, "TestUser");
        let request = Request::new(HelloRequest { message });

        let response = service.unary_rpc(request).await.unwrap().into_inner();
        let decrypted = decrypt_message(&service.aes_key, &response.message);

        assert_eq!(decrypted, "You got, TestUser message!");
    }

    #[tokio::test]
    async fn test_server_streaming_rpc() {
        let service = setup_service();
        let message = encrypt_message(&service.aes_key, "StreamUser");
        let request = Request::new(HelloRequest { message });

        let response = service
            .server_streaming_rpc(request)
            .await
            .unwrap()
            .into_inner();
        let messages: Vec<_> = response
            .map(|res| {
                let msg = res.unwrap().message;
                decrypt_message(&service.aes_key, &msg)
            })
            .collect()
            .await;

        assert_eq!(messages.len(), 3);
        assert!(messages[0].contains("StreamUser"));
    }

    #[tokio::test]
    async fn test_client_streaming_rpc() {
        let service = setup_service();
        let (tx, rx) = mpsc::channel(3);

        let names = vec!["Alice", "Bob", "Charlie"];
        for name in &names {
            let message = encrypt_message(&service.aes_key, name);
            tx.send(Ok(HelloRequest { message })).await.unwrap();
        }
        drop(tx);

        let request = Request::new(ReceiverStream::new(rx));
        let response = service
            .client_streaming_rpc(request)
            .await
            .unwrap()
            .into_inner();
        let decrypted = decrypt_message(&service.aes_key, &response.message);

        assert_eq!(decrypted, "Hello, Alice, Bob, Charlie!");
    }

    #[tokio::test]
    async fn test_bi_directional_streaming_rpc() {
        let service = setup_service();
        let (tx, rx) = mpsc::channel(3);

        let names = vec!["User1", "User2"];
        for name in &names {
            let message = encrypt_message(&service.aes_key, name);
            tx.send(Ok(HelloRequest { message })).await.unwrap();
        }
        drop(tx);

        let request = Request::new(ReceiverStream::new(rx));
        let mut response_stream = service
            .bi_directional_streaming_rpc(request)
            .await
            .unwrap()
            .into_inner();

        let mut received = vec![];
        while let Some(Ok(response)) = response_stream.next().await {
            let decrypted = decrypt_message(&service.aes_key, &response.message);
            received.push(decrypted);
        }

        assert_eq!(received.len(), 2);
        assert_eq!(received[0], "Hello, User1!");
    }
}
