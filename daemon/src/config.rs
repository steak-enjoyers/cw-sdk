use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Bind the TCP server to this host.
    pub host: String,
    /// Bind the TCP server to this port.
    pub port: u16,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 26658,
        }
    }
}
