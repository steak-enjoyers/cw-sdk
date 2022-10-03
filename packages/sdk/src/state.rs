use std::collections::BTreeMap;

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
    pub code_count: u64,
    pub codes: BTreeMap<u64, Vec<u8>>,
}

impl State {
    pub fn store_code(&mut self, wasm_byte_code: Vec<u8>) -> Result<u64, StateError> {
        self.code_count += 1;
        let code_id = self.code_count;
        self.codes.insert(code_id, wasm_byte_code);
        Ok(code_id)
    }

    pub fn query_code(&self, code_id: u64) -> Result<Option<Vec<u8>>, StateError> {
        let wasm_byte_code = self.codes.get(&code_id);
        Ok(wasm_byte_code.cloned())
    }
}

#[derive(Debug, Error)]
pub enum StateError {}
