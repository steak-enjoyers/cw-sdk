use std::fs;
use std::path::Path;

use clap::{Args, Subcommand};
use tracing::error;

use crate::{stringify_pathbuf, ClientConfig};

#[derive(Args)]
pub struct QueryCmd {
    #[clap(subcommand)]
    pub subcommand: QuerySubcmd,

    /// Tendermint RPC endpoint; overrides default value in client config
    #[clap(long)]
    node: Option<String>,
}

#[derive(Subcommand)]
pub enum QuerySubcmd {
    /// Retrieve the wasm byte code corresponding to the given id
    Code {
        /// Code id
        code_id: u64,

        /// Where the byte code is to be downloaded to; default to "$(pwd)/${code_id}.wasm"
        #[clap(long)]
        output_path: Option<String>,
    },
    /// Query metadata of a contract
    Contract {
        /// Contract address
        contract: u64,
    },
    /// Perform a wasm raw query
    WasmRaw {
        /// Contract address
        contract: u64,
        /// The key to be queried in the contract store, in base64 encoding
        key: String,
    },
    /// Perform a wasm smart query
    WasmSmart {
        /// Contract address
        contract: u64,
        /// Query message in JSON format
        msg: String,
    },
}

impl QueryCmd {
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
