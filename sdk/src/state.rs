use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Addr, ContractResult, Empty, Event, Response};
use cosmwasm_vm::testing::{mock_env, mock_info};
use cosmwasm_vm::{
    call_execute, call_instantiate, call_query, Backend, Instance, InstanceOptions, Storage,
};
use thiserror::Error;

use crate::{address, auth, wasm};
use crate::hash::sha256;
use crate::msg::{
    AccountResponse, CodeResponse, ContractResponse, GenesisState, SdkMsg,
    SdkQuery, Tx, WasmRawResponse, WasmSmartResponse,
};
use crate::store::ContractStore;

/// The account type to be stored on-chain
#[cw_serde]
pub struct Account {
    /// The account's secp256k1 public key
    pub pubkey: Binary,
    /// The account's sequence number, used to prevent replay attacks.
    /// The first tx ever to be submitted by the account should come with the sequence of 1.
    pub sequence: u64,
}

/// The code metadata and byte code to be stored on-chain
#[cw_serde]
pub struct Code {
    /// Account who stored the code
    pub creator: Addr,
    /// The wasm byte code
    pub wasm_byte_code: Binary,
}

/// The contract metadata to be stored on-chain
#[cw_serde]
pub struct Contract {
    /// This contract's code id
    pub code_id: u64,
    /// A human readable name for the contract
    pub label: String,
    /// Account who is allowed to migrate the contract
    pub admin: Option<Addr>,
}

/// This is mock, will be replaced by an actual database backend
#[derive(Debug, Default)]
pub struct State {
    /// Identifier of the chain
    pub chain_id: String,

    /// Current block height
    pub height: u64,
    /// The total number of wasm byte codes stored
    pub code_count: u64,
    /// The total number of contracts instantiated
    pub contract_count: u64,

    /// Account address -> Account
    pub accounts: HashMap<Addr, Account>,

    /// Code id -> Code
    pub codes: HashMap<u64, Code>,

    /// Contract address -> Contract
    pub contracts: HashMap<Addr, Contract>,

    /// Contract address -> ContractStore
    pub stores: HashMap<Addr, ContractStore>,
}

// public functions for the state machine
impl State {
    /// Returns ABCI info response.
    ///
    /// For now, our mock storage doesn't provide a method to generate the app hash. Instead, we
    /// simply return `sha256(height)` as a mock app hash.
    pub fn info(&self) -> (u64, Vec<u8>) {
        let app_hash = sha256(&self.height.to_be_bytes());
        (self.height, app_hash)
    }

    /// Run genesis messages. Return app hash.
    /// TODO: Once a staking contract is created, return the genesis validator set as well.
    pub fn init_chain(&mut self, app_state_bytes: &[u8]) -> Result<Vec<u8>, StateError> {
        let GenesisState {
            deployer,
            gen_msgs,
        } = serde_json::from_slice(app_state_bytes)?;

        let deployer_addr = address::validate(&deployer)?;

        for msg in gen_msgs {
            match msg {
                SdkMsg::StoreCode {
                    wasm_byte_code,
                } => {
                    self.store_code(&deployer_addr, wasm_byte_code)?;
                },
                SdkMsg::Instantiate {
                    code_id,
                    msg,
                    funds,
                    label,
                    admin,
                } => {
                    let admin_addr = admin.map(|admin| address::validate(&admin)).transpose()?;
                    self.instantiate_contract(
                        &deployer_addr,
                        code_id,
                        msg.into(),
                        funds,
                        label,
                        admin_addr,
                        AddressGenerator::ByLabel,
                    )?;
                },
                SdkMsg::Execute {
                    contract,
                    msg,
                    funds,
                } => {
                    let contract_addr = address::validate(&contract)?;
                    self.execute_contract(&deployer_addr, contract_addr, msg.into(), funds)?;
                },
                SdkMsg::Migrate {
                    contract,
                    code_id,
                    msg,
                } => {
                    let contract_addr = address::validate(&contract)?;
                    self.migrate_contract(&deployer_addr, contract_addr, code_id, msg.into())?;
                },
            }
        }

        let (_, app_hash) = self.info();
        Ok(app_hash)
    }

    /// Handle ABCI queries. Return query responses as raw binaries.
    pub fn handle_query(&self, query_bytes: &[u8]) -> Result<Vec<u8>, StateError> {
        // deserialize the query from bytes
        let query: SdkQuery = serde_json::from_slice(query_bytes)?;

        match query {
            SdkQuery::Account {
                address,
            } => {
                let addr = address::validate(&address)?;
                serde_json::to_vec(&self.query_account(&addr)?)
            },
            SdkQuery::Code {
                code_id,
            } => serde_json::to_vec(&self.query_code(code_id)?),
            SdkQuery::Contract {
                contract,
            } => {
                let contract_addr = address::validate(&contract)?;
                serde_json::to_vec(&self.query_contract(contract_addr)?)
            },
            SdkQuery::WasmRaw {
                contract,
                key,
            } => {
                let contract_addr = address::validate(&contract)?;
                serde_json::to_vec(&self.query_wasm_raw(contract_addr, key.as_slice())?)
            },
            SdkQuery::WasmSmart {
                contract,
                msg,
            } => {
                let contract_addr = address::validate(&contract)?;
                serde_json::to_vec(&self.query_wasm_smart(contract_addr, msg.as_slice())?)
            },
        }
        .map_err(StateError::from)
    }

    /// Handle transactions. Returns events emitted during transaction executions.
    pub fn handle_tx(&mut self, tx_bytes: &[u8]) -> Result<Vec<Event>, StateError> {
        // deserialize the tx from bytes
        let tx: Tx = serde_json::from_slice(tx_bytes)?;

        // authenticate signature, chain id, sequence, etc.
        let (sender_addr, sender_acct) = auth::authenticate_tx(&tx, self)?;

        // increment the sender's sequence number
        self.accounts.insert(sender_addr.clone(), sender_acct);

        let mut events = vec![];

        // execute messages in the tx in order, and collect all the events emitted
        tx.body
            .msgs
            .into_iter()
            .map(|msg| match msg {
                SdkMsg::StoreCode {
                    wasm_byte_code,
                } => {
                    let event = self.store_code(&sender_addr, wasm_byte_code)?;
                    Ok(vec![event])
                },
                SdkMsg::Instantiate {
                    code_id,
                    msg,
                    funds,
                    label,
                    admin,
                } => {
                    let admin_addr = admin.map(|admin| address::validate(&admin)).transpose()?;
                    self.instantiate_contract(
                        &sender_addr,
                        code_id,
                        msg.into(),
                        funds,
                        label,
                        admin_addr,
                        AddressGenerator::ByIds,
                    )
                },
                SdkMsg::Execute {
                    contract,
                    msg,
                    funds,
                } => {
                    let contract_addr = address::validate(&contract)?;
                    self.execute_contract(&sender_addr, contract_addr, msg.into(), funds)
                },
                SdkMsg::Migrate {
                    contract,
                    code_id,
                    msg,
                } => {
                    let contract_addr = address::validate(&contract)?;
                    self.migrate_contract(&sender_addr, contract_addr, code_id, msg.into())
                },
            })
            .try_for_each(|res| -> Result<_, StateError> {
                events.extend(res?);
                Ok(())
            })?;

        Ok(events)
    }

    /// Commit changes in the cached state into the main application state, and advance block
    /// height by 1. Return the updated block height and app hash.
    ///
    /// TODO: Ideally the state machine maintains a cached state for uncommitted changes separate
    /// from the "main" state, and only commits changes in the cached state into the main state upon
    /// this function call. However for now we don't have such a mechanism implemented.
    pub fn commit(&mut self) -> (u64, Vec<u8>) {
        self.height += 1;
        self.info()
    }
}

impl State {
    fn store_code(
        &mut self,
        sender_addr: &Addr,
        wasm_byte_code: Binary,
    ) -> Result<Event, StateError> {
        let hash = sha256(wasm_byte_code.as_slice());
        let hash_str = hex::encode(&hash);

        // increment code count
        self.code_count += 1;

        // insert code into the map
        let code_id = self.code_count;
        self.codes.insert(
            code_id,
            Code {
                creator: sender_addr.clone(),
                wasm_byte_code,
            },
        );

        Ok(Event::new("store_code")
            .add_attribute("code_id", code_id.to_string())
            .add_attribute("sender", sender_addr)
            .add_attribute("hash", hash_str))
    }

    /// TODO: need to check there is no collision between the contract address and account address
    /// before committing the newly instantiated contract to the store
    #[allow(clippy::too_many_arguments)]
    fn instantiate_contract(
        &mut self,
        sender_addr: &Addr,
        code_id: u64,
        msg: Vec<u8>,
        funds: Vec<Coin>,
        label: String,
        admin_addr: Option<Addr>,
        address_generator: AddressGenerator,
    ) -> Result<Vec<Event>, StateError> {
        if !funds.is_empty() {
            return Err(StateError::FundsUnsupported);
        }

        let backend = wasm::create_backend(ContractStore::new());
        let code = &self.codes[&code_id];
        let mut instance = Instance::from_code(
            &code.wasm_byte_code,
            backend,
            InstanceOptions {
                gas_limit: u64::MAX,
                print_debug: true,
            },
            None,
        )?;
        let result: ContractResult<Response<Empty>> = call_instantiate(
            &mut instance,
            &mock_env(),
            &mock_info(sender_addr.as_str(), &[]),
            &msg,
        )?;

        let Backend {
            storage,
            ..
        } = instance.recycle().unwrap();

        match result {
            ContractResult::Ok(response) => {
                if !response.messages.is_empty() {
                    return Err(StateError::SubmessagesUnsupported);
                }

                // increment contract count
                self.contract_count += 1;

                // generate contract address
                let contract_addr = match address_generator {
                    AddressGenerator::ByLabel => address::derive_from_label(&label)?,
                    AddressGenerator::ByIds => address::derive_from_ids(code_id, self.contract_count)?,
                };

                self.contracts.insert(
                    contract_addr.clone(),
                    Contract {
                        code_id,
                        label,
                        admin: admin_addr,
                    },
                );
                self.stores.insert(contract_addr.clone(), storage);

                // collect the events
                let event = Event::new("instantiate_contract")
                    .add_attribute("sender", sender_addr)
                    .add_attribute("code_id", code_id.to_string())
                    .add_attribute("contract", contract_addr)
                    .add_attributes(response.attributes);

                Ok(prepend(event, response.events))
            },
            ContractResult::Err(err) => Err(StateError::Contract(err)),
        }
    }

    fn execute_contract(
        &mut self,
        sender_addr: &Addr,
        contract_addr: Addr,
        msg: Vec<u8>,
        funds: Vec<Coin>,
    ) -> Result<Vec<Event>, StateError> {
        if !funds.is_empty() {
            return Err(StateError::FundsUnsupported);
        }

        let storage = self
            .stores
            .get(&contract_addr)
            .ok_or_else(|| StateError::contract_not_found(&contract_addr))?
            .clone();
        let contract = &self.contracts[&contract_addr];
        let code = &self.codes[&contract.code_id];
        let backend = wasm::create_backend(storage);
        let mut instance = Instance::from_code(
            &code.wasm_byte_code,
            backend,
            InstanceOptions {
                gas_limit: u64::MAX,
                print_debug: true,
            },
            None,
        )?;
        let result: ContractResult<Response<Empty>> = call_execute(
            &mut instance,
            &mock_env(),
            &mock_info(sender_addr.as_str(), &[]),
            &msg,
        )?;

        let Backend {
            storage,
            ..
        } = instance.recycle().unwrap();

        match result {
            ContractResult::Ok(response) => {
                if !response.messages.is_empty() {
                    return Err(StateError::SubmessagesUnsupported);
                }

                self.stores.insert(contract_addr.clone(), storage);

                // collect the events
                let event = Event::new("execute_contract")
                    .add_attribute("sender", sender_addr)
                    .add_attribute("contract", contract_addr)
                    .add_attributes(response.attributes);

                Ok(prepend(event, response.events))
            },
            ContractResult::Err(err) => Err(StateError::Contract(err)),
        }
    }

    fn migrate_contract(
        &self,
        _sender_addr: &Addr,
        _contract_addr: Addr,
        _code_id: u64,
        _msg: Vec<u8>,
    ) -> Result<Vec<Event>, StateError> {
        Err(StateError::MigrationUnsupported)
    }

    fn query_account(&self, addr: &Addr) -> Result<AccountResponse, StateError> {
        self.accounts
            .get(addr)
            .cloned()
            .map(|account| AccountResponse {
                address: addr.into(),
                pubkey: account.pubkey,
                sequence: account.sequence,
            })
            .ok_or_else(|| StateError::account_not_found(addr))
    }

    fn query_code(&self, code_id: u64) -> Result<CodeResponse, StateError> {
        self.codes
            .get(&code_id)
            .cloned()
            .map(|code| CodeResponse {
                creator: code.creator.into(),
                hash: sha256(&code.wasm_byte_code).into(),
                wasm_byte_code: code.wasm_byte_code,
            })
            .ok_or_else(|| StateError::code_not_found(code_id))
    }

    fn query_contract(&self, contract_addr: Addr) -> Result<ContractResponse, StateError> {
        self.contracts
            .get(&contract_addr)
            .cloned()
            .map(|contract| ContractResponse {
                code_id: contract.code_id,
                label: contract.label.clone(),
                admin: contract.admin.map(String::from),
            })
            .ok_or_else(|| StateError::contract_not_found(contract_addr))
    }

    fn query_wasm_raw(
        &self,
        contract_addr: Addr,
        key: &[u8],
    ) -> Result<WasmRawResponse, StateError> {
        let storage = self
            .stores
            .get(&contract_addr)
            .cloned()
            .ok_or_else(|| StateError::contract_not_found(contract_addr))?;
        let (res, _) = storage.get(key);
        let value = res?;
        Ok(WasmRawResponse {
            value: value.map(Binary),
        })
    }

    fn query_wasm_smart(
        &self,
        contract_addr: Addr,
        msg: &[u8],
    ) -> Result<WasmSmartResponse, StateError> {
        let storage = self
            .stores
            .get(&contract_addr)
            .cloned()
            .ok_or_else(|| StateError::contract_not_found(&contract_addr))?;
        let contract = &self.contracts[&contract_addr];
        let code = &self.codes[&contract.code_id];
        let backend = wasm::create_backend(storage);
        let mut instance = Instance::from_code(
            &code.wasm_byte_code,
            backend,
            InstanceOptions {
                gas_limit: u64::MAX,
                print_debug: true,
            },
            None,
        )?;
        let result = call_query(&mut instance, &mock_env(), msg)?;
        Ok(WasmSmartResponse {
            result,
        })
    }
}

/// Insert an event to the front of an array of events.
/// https://www.reddit.com/r/rust/comments/kul4qz/vec_prepend_insert_from_slice/
fn prepend(event: Event, mut events: Vec<Event>) -> Vec<Event> {
    events.splice(..0, vec![event]);
    events
}

/// Represents which algorithm to use to derive contract addresses during instantiation.
enum AddressGenerator {
    /// Used during chain genesis
    ByLabel,
    /// Used post-genesis
    ByIds,
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error(transparent)]
    Backend(#[from] cosmwasm_vm::BackendError),

    #[error(transparent)]
    Vm(#[from] cosmwasm_vm::VmError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Address(#[from] address::AddressError),

    #[error(transparent)]
    Auth(#[from] auth::AuthError),

    #[error("contract emitted error: {0}")]
    Contract(String),

    #[error("no account found with address {address}")]
    AccountNotFound {
        address: String,
    },

    #[error("no wasm binary code found with id {code_id}")]
    CodeNotFound {
        code_id: u64,
    },

    #[error("no contract found with address {address}")]
    ContractNotFound {
        address: String,
    },

    #[error("contract response includes submessages, which is not supported yet")]
    SubmessagesUnsupported,

    #[error("sending funds when instantiating or executing contracts is not supported yet")]
    FundsUnsupported,

    #[error("migrating contracts is not supported yet")]
    MigrationUnsupported,
}

impl StateError {
    pub fn account_not_found(address: impl Into<String>) -> Self {
        Self::AccountNotFound {
            address: address.into(),
        }
    }

    pub fn code_not_found(code_id: u64) -> Self {
        Self::CodeNotFound {
            code_id,
        }
    }

    pub fn contract_not_found(address: impl Into<String>) -> Self {
        Self::ContractNotFound {
            address: address.into(),
        }
    }
}
