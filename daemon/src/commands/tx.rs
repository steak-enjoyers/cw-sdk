use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{Args, Subcommand};
use tendermint_rpc::{Client, HttpClient, Url};

use cw_sdk::msg::{AccountResponse, SdkMsg, SdkQuery, TxBody};

use crate::query::do_abci_query;
use crate::{print, prompt, ClientConfig, DaemonError, Keyring};

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

        /// A human readable name for the contract
        #[clap(long)]
        label: String,

        /// Coins to be sent along the instantiate message
        #[clap(long)]
        funds: Option<String>,

        /// Contract admin, the account who can migrate the contract
        #[clap(long)]
        admin: Option<String>,
    },
    /// Execute a contract
    Execute {
        /// Contract address
        contract: String,
        /// Execute message in JSON format
        msg: String,

        /// Coins to be sent along the execute message
        #[clap(long)]
        funds: Option<String>,
    },
    /// Migrate an existing contract to a new code id
    Migrate {
        /// Contract address
        contract: String,
        /// Code id which this contract will migrate to
        code_id: u64,
        /// Migrate message in JSON format
        msg: String,
    },
}

impl TxCmd {
    pub async fn run(&self, home_dir: &Path) -> Result<(), DaemonError> {
        if !home_dir.exists() {
            return Err(DaemonError::file_not_found(home_dir)?);
        }

        let client_cfg = ClientConfig::load(home_dir)?;

        let chain_id = self.chain_id.as_ref().unwrap_or(&client_cfg.chain_id);

        let url_str = self.node.as_ref().unwrap_or(&client_cfg.node);
        let url = Url::from_str(url_str)?;
        let client = HttpClient::new(url)?;

        let keyring = Keyring::new(home_dir.join("keys"))?;
        let key = keyring.get(&self.from)?;
        let sender_addr = key.address()?;

        // query the sender's sequence number if not provided
        let sequence = match self.sequence {
            None => {
                let sequence = do_abci_query::<_, AccountResponse>(
                    &client,
                    SdkQuery::Account {
                        address: sender_addr.to_string(),
                    },
                )
                .await
                .map(|res| res.sequence)
                .unwrap_or(0);

                // needs to be 1 greater than the on-chain sequence
                sequence + 1
            },
            Some(sequence) => sequence,
        };

        let msg = match &self.subcommand {
            TxSubcmd::Store {
                wasm_byte_code_path,
            } => {
                // TODO: check whether the file exists
                let wasm_byte_code = fs::read(wasm_byte_code_path)?;
                SdkMsg::StoreCode {
                    wasm_byte_code: wasm_byte_code.into(),
                }
            },
            TxSubcmd::Instantiate {
                code_id,
                msg,
                funds,
                label,
                admin
            } => {
                if funds.is_some() {
                    return Err(DaemonError::unsupported_feature("sending funds"));
                }
                SdkMsg::Instantiate {
                    code_id: *code_id,
                    msg: msg.clone().into_bytes().into(),
                    funds: vec![],
                    label: label.clone(),
                    admin: admin.clone(),
                }
            },
            TxSubcmd::Execute {
                contract,
                msg,
                funds,
            } => {
                if funds.is_some() {
                    return Err(DaemonError::unsupported_feature("sending funds"));
                }
                SdkMsg::Execute {
                    contract: contract.clone(),
                    msg: msg.clone().into_bytes().into(),
                    funds: vec![],
                }
            },
            TxSubcmd::Migrate {
                contract,
                code_id,
                msg,
            } => SdkMsg::Migrate {
                contract: contract.clone(),
                code_id: *code_id,
                msg: msg.clone().into_bytes().into(),
            },
        };

        let body = TxBody {
            sender: sender_addr.into(),
            msgs: vec![msg],
            chain_id: chain_id.into(),
            sequence,
        };

        let tx = key.sign_tx(&body)?;
        let tx_bytes = serde_json::to_vec(&tx)?;

        println!();
        println!("successfully signed tx:");
        println!("-----------------------");
        print::json(&tx)?;
        println!();

        if prompt::confirm("broadcast tx?")? {
            let response = client.broadcast_tx_async(tx_bytes.into()).await?;
            println!();
            print::yaml(&response)?;
        }

        Ok(())
    }
}
