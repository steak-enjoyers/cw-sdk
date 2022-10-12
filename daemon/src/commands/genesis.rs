use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use cw_sdk::hash::sha256;
use serde::Serialize;
use tendermint::genesis::Genesis as TmGenesis;
use tracing::{error, info};

use cw_sdk::msg::{GenesisState, SdkMsg};

use crate::{print, stringify_pathbuf};

#[derive(Args)]
pub struct GenesisCmd {
    #[clap(subcommand)]
    subcommand: GenesisSubcommand,

    /// Path to the Tendermint home directory, where the genesis file is located.
    /// Default to `~/.tendermint`.
    #[clap(long)]
    tendermint_home: Option<PathBuf>,
}

/// NOTE: We do not support migrating contracts in the genesis state
#[derive(Subcommand)]
pub enum GenesisSubcommand {
    /// Set the deployer address to be used for the genesis
    SetDeployer {
        address: String,
    },
    /// Add a "store code" message to the genesis state
    Store {
        /// Path to the wasm byte code
        wasm_byte_code_path: PathBuf,
    },
    /// Add an "instantiate contract" message to the genesis state
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
    /// Add an "execute contract" message to the genesis state
    Execute {
        /// Contract address
        contract: u64,
        /// Execute message in JSON format
        msg: String,

        /// Coins to be sent along the execute message
        #[clap(long)]
        funds: Option<String>,
    },
    /// List all codes in the genesis state
    ListCodes,
    /// List all contracts in the genesis state
    ListContracts,
}

impl GenesisCmd {
    pub fn run(&self) {
        let genesis_path = self
            .tendermint_home
            .clone()
            .unwrap_or_else(|| home::home_dir().unwrap().join(".tendermint"))
            .join("config/genesis.json");

        if !genesis_path.exists() {
            error!("genesis file does not exist: {}", stringify_pathbuf(&genesis_path));
            return;
        }

        let genesis_bytes = fs::read(&genesis_path).unwrap();

        // TODO: If using `tendermint init` command, the `app_state` field is actually missing
        // in the genesis file, which causes the deserialization to fail
        let mut genesis: TmGenesis = serde_json::from_slice(&genesis_bytes).unwrap();

        let mut app_state: GenesisState =
            serde_json::from_value(genesis.app_state.clone()).unwrap_or_default();

        match &self.subcommand {
            GenesisSubcommand::SetDeployer {
                address,
            } => {
                // TODO: validate deployer address
                app_state.deployer = address.clone();
                update_and_write(&mut genesis, &app_state, &genesis_path);
            },
            GenesisSubcommand::Store {
                wasm_byte_code_path,
            } => {
                // TODO: check whether the file exists
                let wasm_byte_code = fs::read(wasm_byte_code_path).unwrap();
                app_state.gen_msgs.push(SdkMsg::StoreCode {
                    wasm_byte_code: wasm_byte_code.into(),
                });
                update_and_write(&mut genesis, &app_state, &genesis_path);
            },
            GenesisSubcommand::Instantiate {
                code_id,
                msg,
                funds,
                label,
                admin,
            } => {
                if funds.is_some() {
                    error!("funds is not supported yet");
                    return;
                }
                app_state.gen_msgs.push(SdkMsg::Instantiate {
                    code_id: *code_id,
                    msg: msg.clone().into_bytes().into(),
                    funds: vec![],
                    label: label.clone(),
                    admin: admin.clone(),
                });
                update_and_write(&mut genesis, &app_state, &genesis_path);
            },
            GenesisSubcommand::Execute {
                contract,
                msg,
                funds
            } => {
                if funds.is_some() {
                    error!("funds is not supported yet");
                    return;
                }
                app_state.gen_msgs.push(SdkMsg::Execute {
                    contract: *contract,
                    msg: msg.clone().into_bytes().into(),
                    funds: vec![],
                });
                update_and_write(&mut genesis, &app_state, &genesis_path);
            },
            GenesisSubcommand::ListCodes => {
                let mut code_count = 0;
                let mut codes = vec![];
                for msg in &app_state.gen_msgs {
                    if let SdkMsg::StoreCode {
                        wasm_byte_code,
                    } = msg
                    {
                        code_count += 1;
                        let hash = sha256(wasm_byte_code.as_slice());
                        codes.push(CodeInfo {
                            code_id: code_count,
                            hash: hex::encode(&hash),
                        });
                    }
                }
                print::yaml(&codes);
            },
            GenesisSubcommand::ListContracts => {
                let mut contract_count = 0;
                let mut contracts = vec![];
                for msg in &app_state.gen_msgs {
                    if let SdkMsg::Instantiate {
                        code_id,
                        label,
                        admin,
                        ..
                    } = msg
                    {
                        contract_count += 1;
                        contracts.push(ContractInfo {
                            address: contract_count,
                            code_id: *code_id,
                            label: label.clone(),
                            admin: admin.clone(),
                        });
                    }
                }
                print::yaml(&contracts);
            },
        }
    }
}

/// Update the genesis state and write to file
fn update_and_write(genesis: &mut TmGenesis, app_state: &GenesisState, genesis_path: &Path) {
    genesis.app_state = serde_json::to_value(app_state).unwrap();
    let genesis_str = serde_json::to_vec_pretty(&genesis).unwrap();
    fs::write(&genesis_path, &genesis_str).unwrap();
    info!("genesis file written to {}", stringify_pathbuf(genesis_path));
}

#[derive(Serialize)]
struct CodeInfo {
    code_id: u64,
    /// SHA-256 hash in hex encoding
    hash: String,
}

#[derive(Serialize)]
struct ContractInfo {
    address: u64,
    code_id: u64,
    label: String,
    admin: Option<String>,
}
