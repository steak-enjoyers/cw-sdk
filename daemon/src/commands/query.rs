use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{Args, Subcommand};
use cosmwasm_std::{ContractResult, Binary};
use serde_json::Value;
use tendermint::abci::transaction::Hash;
use tendermint_rpc::{Client, HttpClient, Url};
use tracing::{info, warn};

use cw_sdk::{
    hash::sha256, AccountResponse, CodeResponse, SdkQuery, WasmRawResponse, WasmSmartResponse,
};

use crate::query::do_abci_query;
use crate::{path, print, ClientConfig, DaemonError};

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
    /// Query a transaction by hash
    Tx {
        /// Transaction hash, in hex encoding
        txhash: String,
    },
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
    /// Perform a wasm raw query
    WasmRaw {
        /// Contract address
        contract: String,
        /// The key to be queried in the contract store, in hex encoding
        key: String,
    },
    /// Perform a wasm smart query
    WasmSmart {
        /// Contract address
        contract: String,
        /// Query message in JSON format
        msg: String,
    },
}

impl QueryCmd {
    pub async fn run(&self, home_dir: &Path) -> Result<(), DaemonError> {
        if !home_dir.exists() {
            return Err(DaemonError::file_not_found(home_dir)?);
        }

        let client_cfg = ClientConfig::load(home_dir)?;
        let url_str = self.node.as_ref().unwrap_or(&client_cfg.node);
        let url = Url::from_str(url_str)?;
        let client = HttpClient::new(url)?;

        match &self.subcommand {
            QuerySubcmd::Tx {
                txhash,
            } => {
                let hash = Hash::from_str(txhash)?;
                let response = client.tx(hash, false).await?;
                print::json(response)?;
            },
            QuerySubcmd::Account {
                address,
            } => {
                let response: AccountResponse = do_abci_query(
                    &client,
                    SdkQuery::Account {
                        address: address.clone(),
                    },
                )
                .await?;

                print::yaml(response)?;
            },
            QuerySubcmd::Code {
                code_id,
                output,
            } => {
                let response: CodeResponse = do_abci_query(
                    &client,
                    SdkQuery::Code {
                        code_id: *code_id,
                    },
                )
                .await?;

                match response.wasm_byte_code {
                    Some(bytes) => {
                        // only print the hash, not the bytecode
                        println!("hash: {}", Binary(sha256(bytes.as_slice())));

                        // save the wasm byte code to file if an output path is specified
                        if let Some(output) = output {
                            fs::write(output, bytes.as_slice())?;
                            info!("wasm byte code written to {}", path::stringify(output)?);
                        }
                    },
                    None => warn!("wasm byte code not found with code id {}", code_id),
                }
            },
            QuerySubcmd::WasmRaw {
                contract,
                key,
            } => {
                let response: WasmRawResponse = do_abci_query(
                    &client,
                    SdkQuery::WasmRaw {
                        contract: contract.clone(),
                        key: hex::decode(key)?.into(),
                    },
                )
                .await?;

                print::yaml(response)?;
            },
            QuerySubcmd::WasmSmart {
                contract,
                msg,
            } => {
                let response: WasmSmartResponse = do_abci_query(
                    &client,
                    SdkQuery::WasmSmart {
                        contract: contract.clone(),
                        msg: msg.clone().into_bytes().into(),
                    },
                )
                .await?;

                match response.result {
                    ContractResult::Ok(bytes) => {
                        // attempt to decode the response as generic JSON
                        match serde_json::from_slice::<Value>(bytes.as_slice()) {
                            Ok(s) => {
                                print::json(s)?;
                            },
                            Err(err) => {
                                println!("query successful but failed to decode response: {err}");
                            },
                        }
                    },
                    ContractResult::Err(err) => println!("query failed: {err}"),
                }
            },
        };

        Ok(())
    }
}
