use cosmwasm_std::{Binary, ContractResult, SystemResult};
use cosmwasm_vm::{Backend, BackendApi, BackendError, BackendResult, GasInfo, Querier, Storage};
use thiserror::Error;

#[derive(Clone, Copy)]
pub struct WasmApi;

impl BackendApi for WasmApi {
    // TODO: currently we just return the utf8 bytes of the string. in the future we should
    // implement proper bech32 decoding.
    fn canonical_address(&self, human: &str) -> BackendResult<Vec<u8>> {
        let bytes = human.as_bytes().to_owned();
        (Ok(bytes), GasInfo::free())
    }

    // TODO: currently we just return the utf8 bytes of the string. in the future we should
    // implement proper bech32 decoding.
    // a question here is, if this function is supposed to be stateless, how do we know which bech32
    // prefix to use? for Go SDK the prefix is hardcoded in the daemon, but for cw-sdk we don't want
    // to hardcode any chain-specific params.
    fn human_address(&self, canonical: &[u8]) -> BackendResult<String> {
        let human = String::from_utf8(canonical.to_owned())
            .map_err(|_| BackendError::user_err("invalid utf8"));
        (human, GasInfo::free())
    }
}

pub struct WasmQuerier;

impl Querier for WasmQuerier {
    fn query_raw(
        &self,
        _request: &[u8],
        _gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        (Err(BackendError::user_err("`querier.query_raw` is not yet implemented")), GasInfo::free())
    }
}

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("contract uses a feature that is not yet implemented: {feature}")]
    Unimplemented {
        feature: String,
    },
}

impl WasmError {
    pub fn unimplemented(feature: impl ToString) -> Self {
        Self::Unimplemented {
            feature: feature.to_string(),
        }
    }
}

pub fn create<T: Storage>(storage: T) -> Backend<WasmApi, T, WasmQuerier> {
    Backend {
        api: WasmApi,
        storage,
        querier: WasmQuerier,
    }
}
