use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{Args, Subcommand};
use tendermint_rpc::{Client, HttpClient, Url};
use tracing::{error, info};

use cw_sdk::msg::{CodeResponse, ContractResponse, SdkQuery, WasmRawResponse, WasmSmartResponse, AccountResponse};

use crate::print::print_as_yaml;
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
    /// Query an account's public key and sequence number
    Account {
        /// Account address
        address: String,
    },
    /// Retrieve the metadata and wasm byte code corresponding to the given code id
    Code {
        /// Code id
        code_id: u64,

        /// If given, then save the wasm byte code to this path
        #[clap(long)]
        output: Option<PathBuf>,
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
        /// The key to be queried in the contract store, in hex encoding
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
    pub async fn run(&self, home_dir: &Path) {
        if !home_dir.exists() {
            error!("home directory does not exist: {}", stringify_pathbuf(home_dir));
            return;
        }

        let client_cfg = ClientConfig::load(home_dir).unwrap();
        let url_str = self.node.as_ref().unwrap_or(&client_cfg.node);
        let url = Url::from_str(url_str).unwrap();
        let client = HttpClient::new(url).unwrap();

        let query = match &self.subcommand {
            QuerySubcmd::Account {
                address,
            } => SdkQuery::Account {
                address: address.clone(),
            },
            QuerySubcmd::Code {
                code_id,
                ..
            } => SdkQuery::Code {
                code_id: *code_id,
            },
            QuerySubcmd::Contract {
                contract,
            } => SdkQuery::Contract {
                contract: *contract,
            },
            QuerySubcmd::WasmRaw {
                contract,
                key,
            } => SdkQuery::WasmRaw {
                contract: *contract,
                key: hex::decode(&key).unwrap().into(),
            },
            QuerySubcmd::WasmSmart {
                contract,
                msg,
            } => SdkQuery::WasmSmart {
                contract: *contract,
                msg: msg.as_bytes().to_vec().into(),
            },
        };
        let query_bytes = serde_json_wasm::to_vec(&query).unwrap();

        let result = client
            .abci_query(None, query_bytes, None, false)
            .await
            .unwrap();

        match &self.subcommand {
            QuerySubcmd::Account {
                ..
            } => {
                let response: AccountResponse = serde_json_wasm::from_slice(&result.value).unwrap();
                print_as_yaml(&response);
            }
            QuerySubcmd::Code {
                output,
                ..
            } => {
                let response: CodeResponse = serde_json_wasm::from_slice(&result.value).unwrap();
                print_as_yaml(&response);

                if let Some(output) = output {
                    let wasm_byte_code = result.value;
                    fs::write(output, wasm_byte_code).unwrap();
                    info!("wasm byte code written to {}", stringify_pathbuf(output));
                }
            },
            QuerySubcmd::Contract {
                ..
            } => {
                let response: ContractResponse = serde_json_wasm::from_slice(&result.value).unwrap();
                print_as_yaml(&response);
            },
            QuerySubcmd::WasmRaw {
                ..
            } => {
                let response: WasmRawResponse = serde_json_wasm::from_slice(&result.value).unwrap();
                print_as_yaml(&response);
            },
            QuerySubcmd::WasmSmart {
                ..
            } => {
                let response: WasmSmartResponse = serde_json_wasm::from_slice(&result.value).unwrap();
                print_as_yaml(&response);
            },
        }
    }
}
