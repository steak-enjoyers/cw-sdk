use std::{path::Path, str::FromStr};

use clap::{Args, Subcommand};
use tendermint::Hash;
use tendermint_rpc::{Client, HttpClient, Url};

use crate::{print, ClientConfig, DaemonError};

#[derive(Args)]
pub struct TendermintCmd {
    #[command(subcommand)]
    pub subcommand: TendermintSubcmd,

    /// Tendermint RPC endpoint; overrides default value in client config
    #[arg(long)]
    node: Option<String>,
}

#[derive(Subcommand)]
pub enum TendermintSubcmd {
    /// Query Tendermint status, including node info, pubkey, latest block hash,
    /// app hash, block height, and time
    Status,

    /// Query information on P2P and other network connections
    NetInfo,

    /// Query a single block by height
    Block {
        /// Block height (default: latest)
        height: Option<u32>,
    },

    /// Query a single block by hash
    BlockByHash {
        /// Block hash, in hex encoding
        hash: String,
    },

    /// Query ABCI results for a single block by height
    BlockResults {
        height: u32,
    },

    /// Query a single transaction by hash
    Tx {
        /// Transaction hash, in hex encoding
        hash: String,
    },
}

impl TendermintCmd {
    pub async fn run(self, home_dir: &Path) -> Result<(), DaemonError> {
        if !home_dir.exists() {
            return Err(DaemonError::file_not_found(home_dir)?);
        }

        let client_cfg = ClientConfig::load(home_dir)?;
        let url_str = self.node.as_ref().unwrap_or(&client_cfg.node);
        let url = Url::from_str(url_str)?;
        let client = HttpClient::new(url)?;

        match self.subcommand {
            TendermintSubcmd::Status => {
                let response = client.status().await?;
                print::json(response)
            },

            TendermintSubcmd::NetInfo => {
                let response = client.net_info().await?;
                print::json(response)
            },

            TendermintSubcmd::Block {
                height,
            } => match height {
                Some(h) => {
                    let response = client.block(h).await?;
                    print::json(response)
                },
                None => {
                    let response = client.latest_block().await?;
                    print::json(response)
                },
            },

            TendermintSubcmd::BlockByHash {
                hash,
            } => {
                let hash = Hash::from_str(&hash)?;
                let response = client.block_by_hash(hash).await?;
                print::json(response)
            },

            TendermintSubcmd::BlockResults {
                height,
            } => {
                let response = client.block_results(height).await?;
                print::json(response)
            },

            TendermintSubcmd::Tx {
                hash,
            } => {
                let hash = Hash::from_str(&hash)?;
                let response = client.tx(hash, false).await?;
                print::json(response)
            },
        }
    }
}
