use cosmwasm_vm::{BackendApi, BackendError, BackendResult, GasInfo};

#[derive(Clone, Copy)]
pub struct WasmApi;

impl BackendApi for WasmApi {
    fn canonical_address(&self, _human: &str) -> BackendResult<Vec<u8>> {
        return (
            Err(BackendError::user_err("`api.canonical_address` is not yet implemented")),
            GasInfo::free(),
        );
    }

    fn human_address(&self, _canonical: &[u8]) -> BackendResult<String> {
        return (
            Err(BackendError::user_err("`api.human_address` is not yet implemented")),
            GasInfo::free(),
        );
    }
}
