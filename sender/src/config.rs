use anyhow::{Error, Result};
use getrandom::fill;
use std::env;

#[derive(Debug)]
pub struct Config {
    target_host: String,
    target_port: u16,
}

impl Config {
    pub fn build() -> Result<Self, Error> {
        let target_host = env::var("TARGET_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let target_port = env::var("TARGET_PORT")
            .unwrap_or_else(|_| "9876".to_string())
            .parse::<u16>()
            .map_err(|e| Error::new(e).context("Failed to parse TARGET_PORT"))?;

        Ok(Config {
            target_host,
            target_port,
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
}
