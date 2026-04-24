use anyhow::{Error, Result};
use std::env;

#[derive(Debug)]
pub struct Config {
    open_port: u16,
    open_host: String,
    db_url: String,
    node_ids: Vec<u16>,
}

impl Config {
    pub fn build() -> Result<Self, Error> {
        let db_url = env::var("DB_URL").unwrap_or_else(|_| "sqlite:receiver.db".to_string());
        let open_host = env::var("DB_URL").unwrap_or_else(|_| "0.0.0.0".to_string());
        let open_port = env::var("OPEN_PORT")
            .unwrap_or_else(|_| "9876".to_string())
            .parse::<u16>()
            .map_err(|e| Error::new(e).context("Failed to parse OPEN_PORT"))?;
        let node_ids = env::var("NODE_IDS")
            .unwrap_or_else(|_| "369,963".to_string())
            .split(',')
            .map(|raw| raw.trim().parse::<u16>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::new(e).context("Failed to pars NODE_IDS"))?;
        Ok(Config {
            open_host,
            open_port,
            db_url,
            node_ids,
        })
    }

    pub fn get_address(&self) -> Result<String, Error> {
        Ok(format!("{}:{}", self.open_host, self.open_port))
    }
    pub fn get_db_url(&self) -> Result<&str, Error> {
        Ok(&self.db_url)
    }
    pub fn get_node_ids(&self) -> &[u16] {
        &self.node_ids
    }
}
