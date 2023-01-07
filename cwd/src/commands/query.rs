use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, UNIX_EPOCH},
};

use chrono::{DateTime, SecondsFormat, Utc};
use clap::{Args, Subcommand};
use cosmwasm_std::{BlockInfo, ContractResult};
use cw_sdk::InfoResponse;
use serde::Serialize;
use serde_json::Value;
use tendermint_rpc::{HttpClient, Url};
use tracing::{error, info};

use cw_sdk::{
    hash::sha256, AccountResponse, CodeResponse, ContractResponse, SdkQuery, WasmRawResponse,
    WasmSmartResponse,
};

use crate::{path, print, query::do_abci_query, ClientConfig, DaemonError};

#[derive(Args)]
pub struct QueryCmd {
    #[command(subcommand)]
    pub subcommand: QuerySubcmd,

    /// Tendermint RPC endpoint; overrides default value in client config
    #[arg(long)]
    node: Option<String>,
}

#[derive(Subcommand)]
pub enum QuerySubcmd {
    /// Query the application's global state
    Info,

    /// Query an account's public key and sequence number
    Account {
        /// Account address
        address: String,
    },

    /// Enumerate all accounts
    Accounts {
        /// Start after this address
        #[arg(long)]
        start_after: Option<String>,

        /// The maximum number of results to be returned in this query
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Query a single contract by label
    Contract {
        /// Contract label
        label: String,
    },

    /// Enumerate all contracts by label
    Contracts {
        /// Start after this contract label
        #[arg(long)]
        start_after: Option<String>,

        /// The maximum number of results to be returned in this query
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Retrieve the metadata and wasm byte code corresponding to the given code id
    Code {
        /// Code id
        code_id: u64,

        /// If given, then save the wasm byte code to this path
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Enumerate all wasm byte codes
    Codes {
        /// Start after this code id
        #[arg(long)]
        start_after: Option<u64>,

        /// The maximum number of results to be returned in this query
        #[arg(long)]
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
            QuerySubcmd::Info => {
                let response: InfoResponse = do_abci_query(
                    &client,
                    SdkQuery::Info {},
                )
                .await?;

                print::json(PrettyInfoResponse::from(response))?;
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

                print::json(response)?;
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
                    },
                )
                .await?;

                print::json(response)?;
            },

            QuerySubcmd::Contract {
                label,
            } => {
                let response: ContractResponse = do_abci_query(
                    &client,
                    SdkQuery::Contract {
                        label,
                    },
                )
                .await?;

                print::json(response)?;
            }

            QuerySubcmd::Contracts {
                start_after,
                limit,
            } => {
                let response: Vec<ContractResponse> = do_abci_query(
                    &client,
                    SdkQuery::Contracts {
                        start_after,
                        limit,
                    },
                )
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
                if let Some(output) = &output {
                    fs::write(output, response.wasm_byte_code.as_slice())?;
                    info!("Wasm byte code written to {}", path::stringify(output)?);
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

                print::json(response)?;
            },

            QuerySubcmd::WasmSmart {
                contract,
                msg,
            } => {
                let response: WasmSmartResponse = do_abci_query(
                    &client,
                    SdkQuery::WasmSmart {
                        contract: contract.clone(),
                        msg: serde_json::from_str(&msg)?,
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
                                error!("Query successful but failed to decode response: {err}");
                            },
                        }
                    },
                    ContractResult::Err(err) => error!("Query failed: {err}"),
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
    hash: String, // hex-encoded SHA-256 hash
}

impl From<&CodeResponse> for HashedCodeResponse {
    fn from(res: &CodeResponse) -> Self {
        Self {
            code_id: res.code_id,
            hash: hex::encode(sha256(&res.wasm_byte_code)),
        }
    }
}

/// Like InfoResponse but BlockInfo is substituted with PrettyBlockInfo.
#[derive(Serialize)]
pub struct PrettyInfoResponse {
    last_committed_block: PrettyBlockInfo,
    code_count: u64,
}

impl From<InfoResponse> for PrettyInfoResponse {
    fn from(res: InfoResponse) -> Self {
        Self {
            last_committed_block: res.last_committed_block.into(),
            code_count: res.code_count,
        }
    }
}

/// The `time` field of BlockTime is serialized into just a number (the UNIX
/// timestamp in nanoseconds) which is not very readable.
/// Here we convert it to a human-readable string according to RFC 3339 standard.
#[derive(Serialize)]
pub struct PrettyBlockInfo {
    height: u64,
    time: String,
    chain_id: String,
}

impl From<BlockInfo> for PrettyBlockInfo {
    fn from(block: BlockInfo) -> Self {
        let d = UNIX_EPOCH + Duration::from_nanos(block.time.nanos());
        let datetime = DateTime::<Utc>::from(d);
        Self {
            height: block.height,
            time: datetime.to_rfc3339_opts(SecondsFormat::Nanos, true),
            chain_id: block.chain_id,
        }
    }
}
