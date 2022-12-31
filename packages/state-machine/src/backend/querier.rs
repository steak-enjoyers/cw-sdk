use cosmwasm_std::{Binary, ContractResult, SystemResult};
use cosmwasm_vm::{BackendError, BackendResult, GasInfo, Querier};

pub struct BackendQuerier;

impl Querier for BackendQuerier {
    fn query_raw(
        &self,
        _request: &[u8],
        _gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        (Err(BackendError::user_err("`querier.query_raw` is not yet implemented")), GasInfo::free())
    }
}
