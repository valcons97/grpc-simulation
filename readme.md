# gRPC mTLS Service with Rust

This project demonstrates how to set up a gRPC service using Rust with mutual TLS (mTLS) for secure communication. It includes instructions for creating the necessary certificates.

## Prerequisites

-   Rust installed on your machine
-   OpenSSL

## Creating Certificates

Follow these steps to create the required certificates:

1.  **Create a Certificate Authority (CA)**

    ```bash
    openssl genrsa -out ca.key 2048
    openssl req -x509 -new -nodes -key ca.key -sha256 -days 1024 -out ca.pem -subj "/CN=localhost"

    ```

2.  **Create a Server Certificate**

    ```bash
    openssl genrsa -out server.key 2048
    openssl req -new -key server.key -out server.csr -subj "/CN=localhost"
    openssl x509 -req -in server.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out server.pem -days 500 -sha256

    ```

3.  **Create a Client Certificate**

    ```bash
    openssl genrsa -out client.key 2048
    openssl req -new -key client.key -out client.csr -subj "/CN=localhost"
    openssl x509 -req -in client.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out client.pem -days 500 -sha256

    ```

4.  **Create a Client CA Certificate**

        ```bash
        cp ca.pem client-ca.pem

        ```

5.  **Create a Client CA Certificate**

        ```bash
        cp ca.pem client-ca.pem

        ```

6.  **Create .env file**

        ```env
        # Paths to the TLS certificates
        SERVER_CERT_PATH=path/to/server.pem
        SERVER_KEY_PATH=path/to/server.key
        CLIENT_CERT_PATH=path/to/client.pem
        CLIENT_KEY_PATH=path/to/client.key
        CLIENT_CA_PATH=path/to/ca.pem

        ```

7.  **Run the server**

        ```bash
        cargo run --bin server

        ```

8.  **Run the client to see the simulation**

        ```bash
        cargo run --bin client

        ```
