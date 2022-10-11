use std::collections::BTreeMap;

use cosmwasm_std::{Binary, ContractResult, Empty, Event, Response};
use cosmwasm_vm::testing::{mock_env, mock_info};
use cosmwasm_vm::{
    call_execute, call_instantiate, call_query, Backend, Instance, InstanceOptions, Storage,
};
use thiserror::Error;

use crate::hash::sha256;
use crate::msg::{
    Account, AccountResponse, CodeResponse, ContractResponse, SdkMsg, SdkQuery, Tx,
    WasmRawResponse, WasmSmartResponse,
};
use crate::store::ContractStore;
use crate::{auth, wasm};

/// The application's state and state transition rules. The core of the blockchain.
#[derive(Debug, Default)]
pub struct State {
    /// Identifier of the chain
    pub chain_id: String,
    /// User accounts: Address -> Account
    /// TODO: use &str instead of String as key?
    pub accounts: BTreeMap<String, Account>,
    /// The total number of wasm byte codes stored
    pub code_count: u64,
    /// Wasm byte codes indexed by the ids
    pub codes: BTreeMap<u64, Vec<u8>>,
    /// The total number of contracts instantiated
    pub contract_count: u64,
    /// The code id used by each contract
    pub contract_codes: BTreeMap<u64, u64>,
    /// Contract store
    pub contract_stores: BTreeMap<u64, ContractStore>,
}

// public functions for the state machine
impl State {
    pub fn handle_tx(&mut self, tx_bytes: &[u8]) -> Result<Vec<Event>, StateError> {
        // deserialize the tx from bytes
        let tx: Tx = serde_json_wasm::from_slice(tx_bytes)?;

        // authenticate signature, chain id, sequence, etc.
        let account = auth::authenticate_tx(&tx, self)?;

        // increment the sender's sequence number
        self.accounts.insert(tx.body.sender.clone(), account);

        let mut events = vec![];

        tx.body
            .msgs
            .into_iter()
            .map(|msg| match msg {
                SdkMsg::StoreCode {
                    wasm_byte_code,
                } => self.store_code(&tx.body.sender, wasm_byte_code.into()),
                SdkMsg::Instantiate {
                    code_id,
                    msg,
                } => self.instantiate_contract(&tx.body.sender, code_id, msg.into()),
                SdkMsg::Execute {
                    contract,
                    msg,
                    funds,
                } => {
                    if !funds.is_empty() {
                        return Err(StateError::FundsUnsupported);
                    }
                    self.execute_contract(&tx.body.sender, contract, msg.into())
                },
                SdkMsg::Migrate {
                    ..
                } => Err(StateError::MigrationUnsupported),
            })
            .try_for_each(|res| -> Result<_, StateError> {
                events.extend(res?);
                Ok(())
            })?;

        Ok(events)
    }

    pub fn handle_query(&self, query_bytes: &[u8]) -> Result<Vec<u8>, StateError> {
        // deserialize the query from bytes
        let query: SdkQuery = serde_json_wasm::from_slice(query_bytes)?;

        match query {
            SdkQuery::Account {
                address,
            } => serde_json_wasm::to_vec(&self.query_account(&address)?),
            SdkQuery::Code {
                code_id,
            } => serde_json_wasm::to_vec(&self.query_code(code_id)?),
            SdkQuery::Contract {
                contract,
            } => serde_json_wasm::to_vec(&self.query_contract(contract)?),
            SdkQuery::WasmRaw {
                contract,
                key,
            } => serde_json_wasm::to_vec(&self.query_wasm_raw(contract, key.as_slice())?),
            SdkQuery::WasmSmart {
                contract,
                msg,
            } => serde_json_wasm::to_vec(&self.query_wasm_smart(contract, msg.as_slice())?),
        }
        .map_err(StateError::from)
    }
}

// private functions for the state machine
impl State {
    fn store_code(
        &mut self,
        sender: &str,
        wasm_byte_code: Vec<u8>,
    ) -> Result<Vec<Event>, StateError> {
        // compute code hash
        let hash = sha256(&wasm_byte_code);

        // increment code count
        self.code_count += 1;

        // insert code into the map
        let code_id = self.code_count;
        self.codes.insert(code_id, wasm_byte_code);

        Ok(vec![Event::new("store_code")
            .add_attribute("code_id", code_id.to_string())
            .add_attribute("sender", sender)
            .add_attribute("hash", hex::encode(&hash))])
    }

    fn instantiate_contract(
        &mut self,
        sender: &str,
        code_id: u64,
        msg: Vec<u8>,
    ) -> Result<Vec<Event>, StateError> {
        let backend = wasm::create_backend(ContractStore::new());
        let mut instance = Instance::from_code(
            &self.codes[&code_id],
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
            &mock_info(sender, &[]),
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

                // for now, we just use a number as contract address
                let contract_addr = self.contract_count;
                self.contract_codes.insert(contract_addr, code_id);

                self.contract_stores.insert(contract_addr, storage);

                // collect the events
                let event = Event::new("instantiate_contract")
                    .add_attribute("sender", sender)
                    .add_attribute("code_id", code_id.to_string())
                    .add_attribute("contract_address", contract_addr.to_string())
                    .add_attributes(response.attributes);

                Ok(prepend(event, response.events))
            },
            ContractResult::Err(err) => Err(StateError::Contract(err)),
        }
    }

    fn execute_contract(
        &mut self,
        sender: &str,
        contract_addr: u64,
        msg: Vec<u8>,
    ) -> Result<Vec<Event>, StateError> {
        let storage = self
            .contract_stores
            .get(&contract_addr)
            .ok_or_else(|| StateError::contract_not_found(contract_addr))?
            .clone();
        let backend = wasm::create_backend(storage);
        let mut instance = Instance::from_code(
            &self.codes[&self.contract_codes[&contract_addr]],
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
            &mock_info(sender, &[]),
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

                self.contract_stores.insert(contract_addr, storage);

                // collect the events
                let event = Event::new("execute_contract")
                    .add_attribute("sender", sender)
                    .add_attribute("contract_address", contract_addr.to_string())
                    .add_attributes(response.attributes);

                Ok(prepend(event, response.events))
            },
            ContractResult::Err(err) => Err(StateError::Contract(err)),
        }
    }

    fn query_account(&self, address: &str) -> Result<AccountResponse, StateError> {
        match self.accounts.get(address) {
            Some(account) => Ok(AccountResponse {
                address: address.into(),
                pubkey: Some(account.pubkey.clone()),
                sequence: account.sequence,
            }),
            None => Ok(AccountResponse {
                address: address.into(),
                pubkey: None,
                sequence: 0,
            }),
        }
    }

    fn query_code(&self, code_id: u64) -> Result<CodeResponse, StateError> {
        match self.codes.get(&code_id) {
            Some(wasm_byte_code) => Ok(CodeResponse {
                hash: sha256(wasm_byte_code).into(),
                wasm_byte_code: wasm_byte_code.clone().into(),
            }),
            None => Err(StateError::code_not_found(code_id)),
        }
    }

    fn query_contract(&self, contract: u64) -> Result<ContractResponse, StateError> {
        match self.contract_codes.get(&contract) {
            Some(code_id) => Ok(ContractResponse {
                code_id: *code_id,
            }),
            None => Err(StateError::contract_not_found(contract)),
        }
    }

    fn query_wasm_raw(&self, contract: u64, key: &[u8]) -> Result<WasmRawResponse, StateError> {
        let storage = self
            .contract_stores
            .get(&contract)
            .ok_or_else(|| StateError::contract_not_found(contract))?
            .clone();
        let (res, _) = storage.get(key);
        let value = res?;
        Ok(WasmRawResponse {
            contract,
            key: key.to_owned().into(),
            value: value.map(Binary),
        })
    }

    fn query_wasm_smart(&self, contract: u64, msg: &[u8]) -> Result<WasmSmartResponse, StateError> {
        let storage = self
            .contract_stores
            .get(&contract)
            .ok_or_else(|| StateError::contract_not_found(contract))?
            .clone();
        let backend = wasm::create_backend(storage);
        let mut instance = Instance::from_code(
            &self.codes[&self.contract_codes[&contract]],
            backend,
            InstanceOptions {
                gas_limit: u64::MAX,
                print_debug: true,
            },
            None,
        )?;
        let result = call_query(&mut instance, &mock_env(), msg)?;
        Ok(WasmSmartResponse {
            contract,
            result,
        })
    }
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error(transparent)]
    Backend(#[from] cosmwasm_vm::BackendError),

    #[error(transparent)]
    Vm(#[from] cosmwasm_vm::VmError),

    #[error(transparent)]
    Serialize(#[from] serde_json_wasm::ser::Error),

    #[error(transparent)]
    Deserialize(#[from] serde_json_wasm::de::Error),

    #[error(transparent)]
    Auth(#[from] auth::AuthError),

    #[error("contract emitted error: {0}")]
    Contract(String),

    #[error("no wasm binary code found with the id {code_id}")]
    CodeNotFound {
        code_id: u64,
    },

    #[error("no contract found under the address {address}")]
    ContractNotFound {
        address: u64,
    },

    #[error("contract response includes submessages, which is not supported yet")]
    SubmessagesUnsupported,

    #[error("sending funds when instantiating or executing contracts is not supported yet")]
    FundsUnsupported,

    #[error("migrating contracts is not supported yet")]
    MigrationUnsupported,
}

impl StateError {
    pub fn code_not_found(code_id: u64) -> Self {
        Self::CodeNotFound {
            code_id,
        }
    }

    pub fn contract_not_found(address: u64) -> Self {
        Self::ContractNotFound {
            address,
        }
    }
}

/// Insert an event to the front of an array of events.
/// https://www.reddit.com/r/rust/comments/kul4qz/vec_prepend_insert_from_slice/
fn prepend(event: Event, mut events: Vec<Event>) -> Vec<Event> {
    events.splice(..0, vec![event]);
    events
}
