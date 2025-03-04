use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use getrandom;

static NONCE_SIZE: usize = 12;

pub struct AesEncryption {
    cipher: Aes256Gcm,
}

impl AesEncryption {
    pub fn new(cipher_key: &str) -> Self {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&cipher_key.as_bytes()));
        Self { cipher }
    }

    pub fn encrypt(&self, plain_text: &str) -> Result<Vec<u8>> {
        // generate random 12 bytes nonce
        let mut nonce = [0u8; NONCE_SIZE];
        getrandom::fill(&mut nonce)
            .map_err(|e| anyhow::anyhow!("getrandom::fill() error, e: {:?}", e))?;
        let nonce = Nonce::from_slice(&nonce);

        // encrypt the plaintext
        let ciphertext = self
            .cipher
            .encrypt(nonce, plain_text.as_bytes())
            .map_err(|e| anyhow::anyhow!("self.cipher.encrypt() error, e: {:?}", e))?;

        // combine nonce and ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, cipher_text: &[u8]) -> Result<String> {
        if cipher_text.len() < NONCE_SIZE {
            return Err(anyhow::anyhow!("ciphertext.len() < {NONCE_SIZE}"));
        }

        // split nonce and ciphertext
        let (nonce, encrypted) = cipher_text.split_at(12);
        let nonce = Nonce::from_slice(nonce);

        // decrypt
        let plain_text = self
            .cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plain_text)
            .map_err(|e| anyhow::anyhow!("String::from_utf8() error, e: {:?}", e))
    }
}
