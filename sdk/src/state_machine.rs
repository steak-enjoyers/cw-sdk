use std::collections::BTreeMap;

use cosmwasm_std::{Binary, ContractResult, Empty, Response, Event};
use cosmwasm_vm::testing::{mock_env, mock_info};
use cosmwasm_vm::{
    call_execute, call_instantiate, call_query, Backend, Instance, InstanceOptions, VmError,
};
use thiserror::Error;

use crate::hash::sha256;
use crate::msg::{Account, CodeResponse, ContractResponse, WasmRawResponse, WasmSmartResponse};
use crate::store::AppStorage;
use crate::wasm;

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
    pub contract_store: AppStorage,
}

impl State {
    pub fn get_chain_id(&self) -> &str {
        &self.chain_id
    }

    pub fn set_chain_id(&mut self, chain_id: String) {
        self.chain_id = chain_id;
    }

    pub fn get_account(&self, address: &str) -> Option<&Account> {
        self.accounts.get(address)
    }

    pub fn set_account(&mut self, address: &str, account: Account) {
        self.accounts.insert(address.to_owned(), account);
    }

    pub fn store_code(&mut self, wasm_byte_code: Vec<u8>) -> Result<u64, StateError> {
        self.code_count += 1;
        let code_id = self.code_count;
        self.codes.insert(code_id, wasm_byte_code);
        Ok(code_id)
    }

    pub fn instantiate_contract(
        &mut self,
        code_id: u64,
        msg: Vec<u8>,
    ) -> Result<(bool, Option<u64>), StateError> {
        let backend = wasm::create_backend(self.contract_store.clone());
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
            &mock_info("larry", &[]),
            &msg,
        )?;

        let Backend {
            storage,
            ..
        } = instance.recycle().unwrap();

        if result.is_ok() {
            // TODO: handle submessages and events emitted by the contract
            self.contract_count += 1;
            let contract_addr = self.contract_count;
            self.contract_codes.insert(contract_addr, code_id);
            self.contract_store = storage;
            Ok((true, Some(contract_addr)))
        } else {
            Ok((false, None))
        }
    }

    pub fn execute_contract(
        &mut self,
        contract_addr: u64,
        msg: Vec<u8>,
    ) -> Result<(bool, Option<Vec<Event>>), StateError> {
        let backend = wasm::create_backend(self.contract_store.clone());
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
            &mock_info("larry", &[]),
            &msg,
        )?;

        let Backend {
            storage,
            ..
        } = instance.recycle().unwrap();

        if result.is_ok() {
            let Response {
                messages,
                mut events,
                attributes,
                ..
            } = result.unwrap();

            // TODO: handle submessages
            // for now we just throw an error if the response includes submessages
            if !messages.is_empty() {
                return Err(StateError::SubmessagesUnsupported);
            }

            // save storage
            self.contract_store = storage;

            // handle events
            let wasm_event = Event::new("wasm").add_attributes(attributes);
            events.push(wasm_event);

            Ok((true, Some(events)))
        } else {
            Ok((false, None))
        }
    }

    pub fn query_code(&self, code_id: u64) -> Result<CodeResponse, StateError> {
        let wasm_byte_code = self.codes.get(&code_id);
        let hash = wasm_byte_code.map(|bytes| sha256(bytes));
        Ok(CodeResponse {
            hash: hash.map(Binary::from),
            wasm_byte_code: wasm_byte_code.cloned().map(Binary::from),
        })
    }

    pub fn query_contract(&self, contract: u64) -> Result<ContractResponse, StateError> {
        match self.contract_codes.get(&contract) {
            Some(code_id) => Ok(ContractResponse {
                code_id: *code_id,
            }),
            None => Err(StateError::contract_not_found(contract)),
        }
    }

    pub fn query_wasm_raw(
        &self,
        contract: u64,
        key: &[u8],
    ) -> Result<WasmRawResponse, StateError> {
        // for now we just dump for whole contract store, regardless of which contract address or
        // key is given. this is for testing purpose
        // need to collect into a Vec first, because serde-json-wasm can't serialize maps
        let data = self
            .contract_store
            .data
            .iter()
            .map(|(key, value)| (Binary(key.clone()), Binary(value.clone())))
            .collect::<Vec<_>>();
        let bytes = serde_json_wasm::to_vec(&data).unwrap();
        Ok(WasmRawResponse {
            contract,
            key: key.to_owned().into(),
            // note again, for testing purpose, this is not the actual key. this is the entire
            // contract store
            value: Some(bytes.into()),
        })
    }

    pub fn query_wasm_smart(
        &self,
        contract: u64,
        msg: &[u8],
    ) -> Result<WasmSmartResponse, StateError> {
        let backend = wasm::create_backend(self.contract_store.clone());
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
    #[error("{0}")]
    Vm(#[from] VmError),

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
