use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use hex;
use rand::Rng;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

pub fn encrypt_message(key: &[u8; 16], plaintext: &str) -> String {
    let iv: [u8; 16] = rand::thread_rng().r#gen();
    let cipher = Aes128Cbc::new_from_slices(key, &iv).unwrap();
    let mut buffer = plaintext.as_bytes().to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0); // Pad buffer

    let ciphertext = cipher.encrypt(&mut buffer, pos).unwrap();
    format!("{}:{}", hex::encode(iv), hex::encode(ciphertext)) // Return IV and ciphertext
}

pub fn decrypt_message(key: &[u8; 16], encrypted: &str) -> String {
    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 2 {
        panic!("Invalid encrypted format");
    }

    let iv = hex::decode(parts[0]).expect("Invalid IV");
    let ciphertext = hex::decode(parts[1]).expect("Invalid ciphertext");

    let cipher = Aes128Cbc::new_from_slices(key, &iv).expect("Invalid key/iv");
    let decrypted = cipher.decrypt_vec(&ciphertext).expect("Decryption failed");
    String::from_utf8(decrypted).expect("Failed to convert to String")
}
