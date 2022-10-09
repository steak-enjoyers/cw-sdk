use std::path::Path;
use std::fs;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Address to listen for ABCI requests
    pub listen_addr: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // including the `tcp://` prefix causes an error...?
            listen_addr: "127.0.0.1:26658".into(),
        }
    }
}

impl AppConfig {
    pub fn load(home_dir: &Path) -> Result<Self, ConfigError> {
        let cfg_path = home_dir.join("config/app.toml");
        let cfg_bytes = fs::read(&cfg_path)?;
        toml::from_slice(&cfg_bytes).map_err(ConfigError::from)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    /// The network chain id
    pub chain_id: String,
    /// Tendermint RPC address for broadcasting transactions for performing queries
    pub node: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            chain_id: "".into(),
            node: "tcp://localhost:26657".into(),
        }
    }
}

impl ClientConfig {
    pub fn load(home_dir: &Path) -> Result<Self, ConfigError> {
        let cfg_path = home_dir.join("config/client.toml");
        let cfg_bytes = fs::read(&cfg_path)?;
        toml::from_slice(&cfg_bytes).map_err(ConfigError::from)
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Deserialize(#[from] toml::de::Error),
}
