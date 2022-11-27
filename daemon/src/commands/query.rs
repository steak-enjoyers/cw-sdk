use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{Args, Subcommand};
use cosmwasm_std::ContractResult;
use cw_sdk::InfoResponse;
use serde::Serialize;
use serde_json::Value;
use tendermint::abci::transaction::Hash;
use tendermint_rpc::{Client, HttpClient, Url};
use tracing::info;

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

    /// Query the blockchain's global state
    Info,

    /// Query an account's public key and sequence number
    Account {
        /// Account address
        address: String,
    },

    /// Enumerate all accounts
    Accounts {
        /// Start after this address
        #[clap(long)]
        start_after: Option<String>,

        /// The maximum number of results to be returned in this query
        #[clap(long)]
        limit: Option<u32>,
    },

    /// Retrieve the metadata and wasm byte code corresponding to the given code id
    Code {
        /// Code id
        code_id: u64,

        /// If given, then save the wasm byte code to this path
        #[clap(long)]
        output: Option<PathBuf>,
    },

    /// Enumerate all wasm byte codes
    Codes {
        /// Start after this code id
        #[clap(long)]
        start_after: Option<u64>,

        /// The maximum number of results to be returned in this query
        #[clap(long)]
        limit: Option<u32>,
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
    pub async fn run(self, home_dir: &Path) -> Result<(), DaemonError> {
        if !home_dir.exists() {
            return Err(DaemonError::file_not_found(home_dir)?);
        }

        let client_cfg = ClientConfig::load(home_dir)?;
        let url_str = self.node.as_ref().unwrap_or(&client_cfg.node);
        let url = Url::from_str(url_str)?;
        let client = HttpClient::new(url)?;

        match self.subcommand {
            QuerySubcmd::Tx {
                txhash,
            } => {
                let hash = Hash::from_str(&txhash)?;
                let response = client.tx(hash, false).await?;
                print::json(response)?;
            },
            QuerySubcmd::Info => {
                let response: InfoResponse = do_abci_query(
                    &client,
                    SdkQuery::Info {},
                )
                .await?;

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
            QuerySubcmd::Accounts {
                start_after,
                limit,
            } => {
                let response: Vec<AccountResponse> = do_abci_query(
                    &client,
                SdkQuery::Accounts {
                    start_after,
                    limit,
                })
                .await?;

                print::json(response)?;
            }
            QuerySubcmd::Code {
                code_id,
                output,
            } => {
                let response: CodeResponse = do_abci_query(
                    &client,
                    SdkQuery::Code {
                        code_id,
                    },
                )
                .await?;

                // only print the hash, not the bytecode
                print::json(HashedCodeResponse::from(&response))?;

                // save the wasm byte code to file if an output path is specified
                if let Some(bytes) = &response.wasm_byte_code {
                    if let Some(output) = &output {
                        fs::write(output, bytes.as_slice())?;
                        info!("wasm byte code written to {}", path::stringify(output)?);
                    }
                }
            },
            QuerySubcmd::Codes {
                start_after,
                limit,
            } => {
                let response = do_abci_query::<_, Vec<CodeResponse>>(
                    &client,
                    SdkQuery::Codes {
                        start_after,
                        limit,
                    },
                )
                .await?
                .iter()
                .map(HashedCodeResponse::from)
                .collect::<Vec<_>>();

                print::json(response)?;
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

/// Just like `CodeResponse` but includes the byte code's hash instead of the
/// full byte code. Used for CLI output.
#[derive(Serialize)]
pub struct HashedCodeResponse {
    code_id: u64,
    /// Hex-encoded SHA-256 hash; None if no code is found under the code id.
    hash: Option<String>,
}

impl From<&CodeResponse> for HashedCodeResponse {
    fn from(res: &CodeResponse) -> Self {
        Self {
            code_id: res.code_id,
            hash: res.wasm_byte_code.as_ref().map(|bytes| hex::encode(sha256(&bytes))),
        }
    }
}
