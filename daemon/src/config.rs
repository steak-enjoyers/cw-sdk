use serde::{Deserialize, Serialize};

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
