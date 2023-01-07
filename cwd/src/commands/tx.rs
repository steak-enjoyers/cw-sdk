use std::{
    any::type_name,
    fs,
    path::{Path, PathBuf},
};

use clap::{Args, Subcommand};
use colored::*;
use cosmwasm_std::Addr;
use cw_sdk::{Account, AccountResponse, SdkMsg, SdkQuery, TxBody};
use tendermint_rpc::Client;
use tracing::warn;

use crate::{
    client::{create_http_client, do_abci_query},
    print, prompt, ClientConfig, DaemonError, Keyring,
};

#[derive(Args)]
pub struct TxCmd {
    #[command(subcommand)]
    pub subcommand: TxSubcmd,

    /// Name of the key which will sign the transaction
    #[arg(long)]
    from: String,

    /// Chain id; overrides default value in client config
    #[arg(long)]
    chain_id: Option<String>,

    /// Sequence number of the signing account
    #[arg(long)]
    sequence: Option<u64>,

    /// Tendermint RPC endpoint; overrides default value in client config
    #[arg(long)]
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
        #[arg(long)]
        label: String,

        /// Coins to be sent along the instantiate message
        #[arg(long)]
        funds: Option<String>,

        /// Contract admin, the account who can migrate the contract
        #[arg(long)]
        admin: Option<String>,
    },

    /// Execute a contract
    Execute {
        /// Contract address
        contract: String,
        /// Execute message in JSON format
        msg: String,

        /// Coins to be sent along the execute message
        #[arg(long)]
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
    pub async fn run(self, home_dir: &Path) -> Result<(), DaemonError> {
        // load sender key
        let keyring = Keyring::new(home_dir.join("keys"))?;
        let key = keyring.get(&self.from)?;
        let sender_addr = key.address()?;

        // create tendermint client
        let client_cfg = ClientConfig::load(home_dir)?;
        let client = create_http_client(self.node.as_ref(), &client_cfg)?;

        // find chain id
        let chain_id = self.chain_id.as_ref().unwrap_or(&client_cfg.chain_id);

        // query the sender's sequence number if not provided
        let sequence = match self.sequence {
            None => {
                let result = do_abci_query::<_, AccountResponse>(
                    &client,
                    SdkQuery::Account {
                        address: sender_addr.to_string(),
                    },
                )
                .await;

                let sequence = match result {
                    // if the account exists and is a base account, we take the
                    // sequence number
                    Ok(AccountResponse {
                        account: Account::Base {
                            sequence,
                            ..
                        },
                        ..
                    }) => sequence,

                    // if the account exists but is a contract, we throw error
                    // because contracts can't sign txs
                    Ok(AccountResponse {
                        account: Account::Contract {
                            ..
                        },
                        ..
                    }) => return Err(DaemonError::sender_is_contract(&sender_addr)),

                    // if query results in an error, and the error is that the
                    // account is not found, we use zero.
                    // the first tx ever to be submitted should have the
                    // sequence of 1.
                    //
                    // TODO: instead of string matching, we should establish a
                    // standardized list of error codes and match the code instead
                    Err(DaemonError::QueryFailed {
                        err,
                    }) if err.contains(&format!("{} not found", type_name::<Account<Addr>>())) => {
                        warn!(
                            "Account with address {} not found on chain. Use default sequence number of 1",
                            &sender_addr,
                        );
                        0
                    },

                    // for other errors, we cannot handle them here, so we throw
                    Err(err) => return Err(err),
                };

                // needs to be 1 greater than the on-chain sequence
                sequence + 1
            },
            Some(sequence) => sequence,
        };

        let msg = match self.subcommand {
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
                admin,
            } => {
                if funds.is_some() {
                    return Err(DaemonError::unsupported_feature("sending funds"));
                }
                SdkMsg::Instantiate {
                    code_id,
                    msg: serde_json::from_str(&msg)?,
                    funds: vec![],
                    label,
                    admin,
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
                    contract,
                    msg: serde_json::from_str(&msg)?,
                    funds: vec![],
                }
            },

            TxSubcmd::Migrate {
                contract,
                code_id,
                msg,
            } => SdkMsg::Migrate {
                contract,
                code_id,
                msg: serde_json::from_str(&msg)?,
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

        println!("{}", "🤖 Transaction signed:".bold());
        print::json(&tx)?;

        if prompt::confirm(format!("{}", "🤔 Broadcast?".bold()))? {
            let response = client.broadcast_tx_async(tx_bytes).await?;
            print::json(response)?;
            println!("{}", "🙌 Successfully broadcasted!".bold());
        }

        Ok(())
    }
}
