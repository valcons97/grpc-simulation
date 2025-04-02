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
    buffer.resize(pos + 16, 0);

    let ciphertext = cipher.encrypt(&mut buffer, pos).unwrap();
    format!("{}:{}", hex::encode(iv), hex::encode(ciphertext))
}

pub fn decrypt_message(key: &[u8; 16], encrypted: &str) -> String {
    let parts: Vec<&str> = encrypted.split(':').collect();
    let iv = hex::decode(parts[0]).unwrap();
    let ciphertext = hex::decode(parts[1]).unwrap();

    let cipher = Aes128Cbc::new_from_slices(key, &iv).unwrap();
    let decrypted = cipher.decrypt_vec(&ciphertext).unwrap();
    String::from_utf8(decrypted).unwrap()
}
