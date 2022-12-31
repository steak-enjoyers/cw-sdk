use cosmwasm_vm::{BackendResult, GasInfo};

use cw_sdk::address;

use super::into_backend_err;

#[derive(Clone, Copy)]
pub struct BackendApi;

impl cosmwasm_vm::BackendApi for BackendApi {
    fn canonical_address(&self, human: &str) -> BackendResult<Vec<u8>> {
        let bytes = address::canonicalize(human)
            .map(|addr| addr.to_vec())
            .map_err(into_backend_err);
        (bytes, GasInfo::free())
    }

    fn human_address(&self, canonical: &[u8]) -> BackendResult<String> {
        let human = address::humanize(&canonical.into())
            .map(String::from)
            .map_err(into_backend_err);
        (human, GasInfo::free())
    }
}
