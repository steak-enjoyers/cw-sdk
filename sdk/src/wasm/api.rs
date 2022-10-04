use cosmwasm_vm::{BackendApi, BackendError, BackendResult, GasInfo};

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
