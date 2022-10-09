use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use tracing::error;

use crate::{stringify_pathbuf, ClientConfig};

#[derive(Args)]
pub struct TxCmd {
    #[clap(subcommand)]
    pub subcommand: TxSubcmd,

    /// Name of the key which will sign the transaction
    #[clap(long)]
    from: String,

    /// Chain id; overrides default value in client config
    #[clap(long)]
    chain_id: Option<String>,

    /// Sequence number of the signing account
    #[clap(long)]
    sequence: Option<u64>,

    /// Tendermint RPC endpoint; overrides default value in client config
    #[clap(long)]
    node: Option<String>,
}

#[derive(Subcommand)]
pub enum TxSubcmd {
    /// Upload wasm byte code
    Store {
        /// Path to the wasm byte code
        wasm_byte_code_path: PathBuf,
    },
    /// Instantiate a new contract
    Instantiate {
        /// Code id
        code_id: u64,
        /// Instantiate message in JSON format
        msg: String,
    },
    /// Execute a contract
    Execute {
        /// Contract address
        contract: u64,
        /// Execute message in JSON format
        msg: String,
    },
    /// Migrate an existing contract to a new code id
    Migrate {
        /// Contract address
        contract: u64,
        /// Code id which this contract will migrate to
        code_id: u64,
    },
}

impl TxCmd {
    pub fn run(&self, home_dir: &Path) {
        if !home_dir.exists() {
            error!("home directory does not exist: {}", stringify_pathbuf(home_dir));
            return;
        }

        let client_cfg_path = home_dir.join("config/client.toml");
        let client_cfg_bytes = fs::read(&client_cfg_path).unwrap();
        let client_cfg: ClientConfig = serde_json_wasm::from_slice(&client_cfg_bytes).unwrap();
        dbg!(&client_cfg);

        error!("unimplemented");
    }
}
