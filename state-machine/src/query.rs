use cosmwasm_std::{testing::mock_env, Binary, Storage};
use cosmwasm_vm::{call_query, Backend, Instance, InstanceOptions, Storage as VmStorage};
use cw_storage_plus::Bound;

use cw_sdk::{
    address, paginate_map, AccountResponse, CodeResponse, InfoResponse, WasmRawResponse,
    WasmSmartResponse,
};

use crate::{
    backend::{BackendApi, BackendQuerier, ContractSubstore},
    error::Result,
    state::{code_by_address, ACCOUNTS, BLOCK_HEIGHT, CHAIN_ID, CODES, CODE_COUNT, CONTRACT_COUNT},
};

pub fn info(store: &dyn Storage) -> Result<InfoResponse> {
    Ok(InfoResponse {
        chain_id: CHAIN_ID.load(store)?,
        height: BLOCK_HEIGHT.load(store)?,
        code_count: CODE_COUNT.load(store)?,
        contract_count: CONTRACT_COUNT.load(store)?,
    })
}

pub fn account(store: &dyn Storage, address: String) -> Result<AccountResponse> {
    let addr = address::validate(&address)?;
    let opt = ACCOUNTS.may_load(store, &addr)?;
    Ok(AccountResponse {
        address,
        account: opt.map(|account| account.into()),
    })
}

pub fn accounts(
    store: &dyn Storage,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<AccountResponse>> {
    let start = start_after.map(|address| Bound::ExclusiveRaw(address.into_bytes()));
    paginate_map(ACCOUNTS, store, start, limit, |item| {
        let (address, account) = item?;
        Ok(AccountResponse {
            address: address.into(),
            account: Some(account.into()),
        })
    })
}

pub fn code(store: &dyn Storage, code_id: u64) -> Result<CodeResponse> {
    Ok(CodeResponse {
        code_id,
        wasm_byte_code: CODES.may_load(store, code_id)?,
    })
}

pub fn codes(
    store: &dyn Storage,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> Result<Vec<CodeResponse>> {
    let start = start_after.map(Bound::exclusive);
    paginate_map(CODES, store, start, limit, |item| {
        let (code_id, bytes) = item?;
        Ok(CodeResponse {
            code_id,
            wasm_byte_code: Some(bytes),
        })
    })
}

pub fn wasm_raw(store: impl Storage, contract: &str, key: &[u8]) -> Result<WasmRawResponse> {
    let contract_addr = address::validate(contract)?;
    let substore = ContractSubstore::new(store, &contract_addr);
    let (value, _) = substore.get(key);
    Ok(WasmRawResponse {
        value: value?.map(Binary),
    })
}

pub fn wasm_smart(store: impl Storage, contract: &str, msg: &[u8]) -> Result<WasmSmartResponse> {
    let contract_addr = address::validate(contract)?;
    let code = code_by_address(&store, &contract_addr)?;

    let mut instance = Instance::from_code(
        &code,
        Backend {
            api: BackendApi,
            storage: ContractSubstore::new(store, &contract_addr),
            querier: BackendQuerier,
        },
        InstanceOptions {
            gas_limit: u64::MAX,
            print_debug: true,
        },
        None,
    )?;

    let result = call_query(&mut instance, &mock_env(), msg)?;

    Ok(WasmSmartResponse {
        result,
    })
}
