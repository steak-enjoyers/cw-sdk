use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::fs;

use clap::{Args, Subcommand};
use dialoguer::Confirm;
use tendermint_rpc::{Client, HttpClient, Url};
use tracing::{error, info};

use cw_sdk::auth::ACCOUNT_PREFIX;
use cw_sdk::msg::{SdkQuery, SdkMsg, AccountResponse, TxBody};

use crate::print::print_as_json;
use crate::{stringify_pathbuf, ClientConfig, Keyring};

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
        /// Migrate message in JSON format
        msg: String,
    },
}

impl TxCmd {
    pub async fn run(&self, home_dir: &Path) {
        if !home_dir.exists() {
            error!("home directory does not exist: {}", stringify_pathbuf(home_dir));
            return;
        }

        let client_cfg = ClientConfig::load(home_dir).unwrap();

        let chain_id = self.chain_id.as_ref().unwrap_or(&client_cfg.chain_id);

        let url_str = self.node.as_ref().unwrap_or(&client_cfg.node);
        let url = Url::from_str(url_str).unwrap();
        let client = HttpClient::new(url).unwrap();

        let keyring = Keyring::new(home_dir.join("keys")).unwrap();
        let key = keyring.get(&self.from).unwrap();
        let sender = key.address().bech32(ACCOUNT_PREFIX).unwrap();

        // query the sender's sequence number if not provided
        let sequence = match self.sequence {
            None => {
                let query = SdkQuery::Account {
                    address: sender.clone(),
                };
                let query_bytes = serde_json_wasm::to_vec(&query).unwrap();
                let result = client
                    .abci_query(None, query_bytes, None, false)
                    .await
                    .unwrap();
                let response: AccountResponse = serde_json_wasm::from_slice(&result.value).unwrap();
                // needs to be 1 greater than the on-chain sequence
                response.sequence + 1
            },
            Some(sequence) => sequence,
        };

        let msg = match &self.subcommand {
            TxSubcmd::Store {
                wasm_byte_code_path,
            } => {
                let wasm_byte_code = fs::read(wasm_byte_code_path).unwrap();
                SdkMsg::StoreCode {
                    wasm_byte_code: wasm_byte_code.into(),
                }
            },
            TxSubcmd::Instantiate {
                code_id,
                msg,
            } => SdkMsg::Instantiate {
                code_id: *code_id,
                msg: msg.clone().into_bytes().into(),
            },
            TxSubcmd::Execute {
                contract,
                msg,
            } => SdkMsg::Execute {
                contract: *contract,
                msg: msg.clone().into_bytes().into(),
                funds: vec![],
            },
            TxSubcmd::Migrate {
                contract,
                code_id,
                msg,
            } => SdkMsg::Migrate {
                contract: *contract,
                code_id: *code_id,
                msg: msg.clone().into_bytes().into(),
            },
        };

        let body = TxBody {
            sender,
            msgs: vec![msg],
            chain_id: chain_id.into(),
            sequence
        };

        let tx = key.sign_tx(&body).unwrap();
        let tx_bytes = serde_json_wasm::to_vec(&tx).unwrap();

        info!("signed tx using key {}", key.name);
        println!();
        print_as_json(&tx);
        println!();

        if Confirm::new().with_prompt("broadcast tx?").interact().unwrap() {
            client.broadcast_tx_async(tx_bytes.into()).await.unwrap();
        }
    }
}
