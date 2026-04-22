use anyhow::{Error, Result};
use std::env;

#[derive(Debug)]
pub struct Config {
    open_port: u16,
}

impl Config {
    pub fn build() -> Result<Self, Error> {
        let open_port = env::var("OPEN_PORT")
            .unwrap_or_else(|_| "9876".to_string())
            .parse::<u16>()
            .map_err(|e| Error::new(e).context("Failed to parse OPEN_PORT"))?;

        Ok(Config { open_port })
    }

    pub fn get_address(&self) -> Result<String, Error> {
        Ok(format!("127.0.0.1:{}", self.open_port))
    }
}
