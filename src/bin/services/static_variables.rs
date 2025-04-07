use std::convert::TryInto;
use std::env;

#[derive(Debug)]
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
        dotenv::dotenv().ok();

        Self {
            cert_file_path: load_env("CLIENT_CERT_PATH"),
            server_root_file_path: load_env("SERVER_ROOT_PATH"),
            key_file_path: load_env("CLIENT_KEY_PATH"),
            server_url: load_env("SERVER_URL"),
            server_domain_name: load_env("SERVER_DOMAIN_NAME"),
            server_cert_path: load_env("SERVER_CERT_PATH"),
            server_key_path: load_env("SERVER_KEY_PATH"),
            server_client_path: load_env("SERVER_CLIENT_PATH"),
            aes_key: load_aes_key("AES_KEY"),
        }
    }
}

fn load_env(var_name: &str) -> String {
    env::var(var_name).expect(&format!("{} not set", var_name))
}

fn load_aes_key(var_name: &str) -> [u8; 16] {
    let aes_key_str = env::var(var_name).expect(&format!("{} not set", var_name));
    let aes_key: Vec<u8> = aes_key_str.as_bytes().to_vec();

    if aes_key.len() != 16 {
        panic!("AES key must be 16 bytes long.");
    }

    aes_key.try_into().expect("AES key conversion failed")
}
