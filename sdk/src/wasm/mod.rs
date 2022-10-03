mod api;
mod error;
mod querier;
mod storage;

pub use api::WasmApi;
use cosmwasm_vm::Backend;
pub use error::WasmError;
pub use querier::WasmQuerier;
pub use storage::WasmStorage;

pub fn create_backend(storage: &WasmStorage) -> Backend<WasmApi, WasmStorage, WasmQuerier> {
    Backend {
        api: WasmApi,
        storage: storage.clone(),
        querier: WasmQuerier,
    }
}
