use std::env;

pub struct Global {
    pub cert_file_path: String,
    pub server_root_file_path: String,
    pub key_file_path: String,
    pub server_url: String,
    pub server_domain_name: String,
    pub server_cert_path: String,
    pub server_key_path: String,
    pub server_client_path: String,
    pub aes_key: [u8; 16],
}

impl Global {
    pub fn new() -> Self {
        let cert_file_path = env::var("CLIENT_CERT_PATH").expect("CLIENT_CERT_PATH not set");
        let server_root_file_path = env::var("SERVER_ROOT_PATH").expect("SERVER_ROOT_PATH not set");
        let key_file_path = env::var("CLIENT_KEY_PATH").expect("CLIENT_KEY_PATH not set");
        let server_url = env::var("SERVER_URL").expect("SERVER_URL not set");
        let server_domain_name =
            env::var("SERVER_DOMAIN_NAME").expect("SERVER_DOMAIN_NAME not set");

        let server_cert_path = env::var("SERVER_CERT_PATH").expect("SERVER_CERT_PATH not set");
        let server_key_path = env::var("SERVER_KEY_PATH").expect("SERVER_KEY_PATH not set");
        let server_client_path =
            env::var("SERVER_CLIENT_PATH").expect("SERVER_CLIENT_PATH not set");

        let aes_key_str = env::var("AES_KEY").expect("AES_KEY not set");
        let aes_key: Vec<u8> = aes_key_str.as_bytes().to_vec();

        if aes_key.len() != 16 {
            panic!("AES key must be 16 bytes long.");
        }

        let aes_key: [u8; 16] = aes_key.try_into().expect("AES key conversion failed");

        Global {
            cert_file_path,
            server_root_file_path,
            key_file_path,
            server_url,
            server_domain_name,
            server_cert_path,
            server_key_path,
            server_client_path,
            aes_key,
        }
    }
}
