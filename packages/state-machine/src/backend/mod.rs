mod api;
mod querier;
mod storage;

pub use api::BackendApi;
pub use querier::BackendQuerier;
pub use storage::ContractSubstore;

use cosmwasm_vm::BackendError;

fn into_backend_err(err: impl std::error::Error) -> BackendError {
    BackendError::user_err(err.to_string())
}
