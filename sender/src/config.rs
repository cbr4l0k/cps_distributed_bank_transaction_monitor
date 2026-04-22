use anyhow::{Error, Result, anyhow};
use getrandom::fill;
use std::env;

#[derive(Debug)]
pub struct Config {
    target_host: String,
    target_port: u16,
    key: [u8; 32],
}

impl Config {
    pub fn build() -> Result<Self, Error> {
        let key: [u8; 32];
        match env::var("CIPHER_KEY") {
            Ok(data) => {
                let key_bytes: [u8; 32] = hex::decode(data)?
                    .try_into()
                    .map_err(|v: Vec<u8>| anyhow!("expected 32 bytes, got {}", v.len()))?;
                key = key_bytes;
            }
            Err(_) => return Err(anyhow!("no CIPHER_KEY env detected :(")),
        }
        let target_host = env::var("TARGET_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let target_port = env::var("TARGET_PORT")
            .unwrap_or_else(|_| "9876".to_string())
            .parse::<u16>()
            .map_err(|e| Error::new(e).context("Failed to parse TARGET_PORT"))?;

        Ok(Config {
            target_host,
            target_port,
            key,
        })
    }

    pub fn get_address(&self) -> Result<String, Error> {
        Ok(format!("{}:{}", self.target_host, self.target_port))
    }

    pub fn get_random_nonce() -> Result<[u8; 12], Error> {
        let mut nonce = [0u8; 12];
        fill(&mut nonce)?;
        Ok(nonce)
    }

    pub fn get_cipher_key(&self) -> Result<&[u8; 32], Error> {
        Ok(&self.key)
    }
}
