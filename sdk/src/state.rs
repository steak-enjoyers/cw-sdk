use std::collections::BTreeMap;

use cosmwasm_std::{Empty, ContractResult, Response};
use cosmwasm_vm::{
    call_instantiate,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Backend, Instance, InstanceOptions, VmError, Storage, call_query,
};
use thiserror::Error;

/// The application's state and state transition rules. The core of the blockchain.
///
/// Currently we use an in-memory state for development. For an actually usable blockchain, it is
/// necessary to switch to a persistent database backend. Two options are Nomic's Merk tree or
/// Penumbra's Jellyfish Merkle tree (JMT):
/// - https://twitter.com/zmanian/status/1576643740784947200
/// - https://developers.diem.com/papers/jellyfish-merkle-tree/2021-01-14.pdf
/// - https://github.com/penumbra-zone/jmt
#[derive(Debug, Default)]
pub struct State {
    /// The total number of wasm byte codes stored
    pub code_count: u64,
    /// Wasm byte codes indexed by the ids
    pub codes: BTreeMap<u64, Vec<u8>>,
    /// The total number of contracts instantiated
    pub contract_count: u64,
    /// The code id used by each contract
    pub contract_codes: BTreeMap<u64, u64>,
    /// Contract stores
    pub contract_stores: BTreeMap<u64, MockStorage>,
}

impl State {
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
        let backend = Backend {
            api: MockApi::default(),
            storage: MockStorage::default(),
            querier: MockQuerier::<Empty>::new(&[]),
        };
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
            self.contract_count += 1;
            let contract_addr = self.contract_count;
            self.contract_codes.insert(contract_addr, code_id);
            self.contract_stores.insert(contract_addr, storage);
            Ok((true, Some(contract_addr)))
        } else {
            Ok((false, None))
        }
    }

    pub fn query_code(&self, code_id: u64) -> Result<Option<Vec<u8>>, StateError> {
        let wasm_byte_code = self.codes.get(&code_id);
        Ok(wasm_byte_code.cloned())
    }

    pub fn query_wasm_raw(
        &self,
        contract_addr: u64,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, StateError> {
        let store = &self.contract_stores[&contract_addr];
        let (res, _) = store.get(key);
        Ok(res.unwrap())
    }

    pub fn query_wasm_smart(
        &self,
        contract_addr: u64,
        msg: &[u8]
    ) -> Result<(bool, Option<Vec<u8>>), StateError> {
        let backend = Backend {
            api: MockApi::default(),
            storage: self.contract_stores[&contract_addr].clone(), // fuck, mock storage doesn't have clone
            querier: MockQuerier::<Empty>::new(&[]),
        };
        let mut instance = Instance::from_code(
            &self.codes[&self.contract_codes[&contract_addr]],
            backend,
            InstanceOptions {
                gas_limit: u64::MAX,
                print_debug: true,
            },
            None,
        )?;
        let result = call_query(&mut instance, &mock_env(), msg)?;

        if result.is_ok() {
            Ok((true, Some(result.unwrap().0)))
        } else {
            Ok((false, None))
        }
    }
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("{0}")]
    Vm(#[from] VmError),
}
