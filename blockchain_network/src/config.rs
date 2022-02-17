use openssl::{
    rsa::{Padding, Rsa},
    symm::Cipher,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub address: String,
    pub private_key_pem: String,
    pub public_key_pem: String,
    pub passphrase: String,
}

impl Config {
    pub fn new(address: String, passphrase: String) -> Self {
        let rsa = Rsa::generate(1024).unwrap();
        Config {
            address,
            private_key_pem: String::from_utf8(
                rsa.private_key_to_pem_passphrase(
                    Cipher::aes_128_cbc(),
                    passphrase.clone().as_bytes(),
                )
                .unwrap(),
            )
            .unwrap(),
            public_key_pem: String::from_utf8(rsa.public_key_to_pem().unwrap()).unwrap(),
            passphrase,
        }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let rsa = Rsa::public_key_from_pem(self.public_key_pem.as_bytes()).unwrap();
        let mut buffer: Vec<u8> = vec![0; rsa.size() as usize];
        let _ = rsa
            .public_encrypt(data, &mut buffer, Padding::PKCS1)
            .unwrap();
        buffer
    }

    pub fn decrypt(&self, encrypted_data: Vec<u8>) -> String {
        let rsa = Rsa::private_key_from_pem_passphrase(
            self.private_key_pem.as_bytes(),
            self.passphrase.as_bytes(),
        )
        .unwrap();
        let mut buffer: Vec<u8> = vec![0; rsa.size() as usize];
        let _ = rsa
            .private_decrypt(&encrypted_data, &mut buffer, Padding::PKCS1)
            .unwrap();
        let decrypted_data = String::from_utf8(buffer.clone()).unwrap();
        let decrypted_data = decrypted_data.trim_matches(char::from(0));
        decrypted_data.to_string()
    }
}
