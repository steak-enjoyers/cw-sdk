mod api;
mod error;
mod querier;

pub use api::WasmApi;
use cosmwasm_vm::{Backend, Storage};
pub use error::WasmError;
pub use querier::WasmQuerier;

pub fn create_backend<T: Storage>(storage: T) -> Backend<WasmApi, T, WasmQuerier> {
    Backend {
        api: WasmApi,
        storage,
        querier: WasmQuerier,
    }
}
