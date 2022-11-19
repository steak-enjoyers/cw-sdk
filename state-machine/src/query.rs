use cosmwasm_std::{testing::mock_env, Binary, Storage};
use cosmwasm_vm::{call_query, Backend, Instance, InstanceOptions, Storage as BackendStorage};
use cw_sdk::{
    address, Account, AccountResponse, CodeResponse, InfoResponse, WasmRawResponse,
    WasmSmartResponse,
};
use cw_storage_plus::Bound;

use crate::{
    backend::{backend_store_read, BackendApi, BackendQuerier},
    error::{Error, Result},
    state::{ACCOUNTS, CHAIN_ID, CODES, CODE_COUNT, CONTRACT_COUNT, HEIGHT},
};

use cw_sdk::paginate_map;

pub fn info(store: &dyn Storage) -> Result<InfoResponse> {
    let chain_id = CHAIN_ID.load(store)?;
    let height = HEIGHT.load(store)?;
    let code_count = CODE_COUNT.load(store)?;
    let contract_count = CONTRACT_COUNT.load(store)?;
    Ok(InfoResponse {
        chain_id,
        height,
        code_count,
        contract_count,
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
    let wasm_byte_code = CODES.may_load(store, code_id)?;
    Ok(CodeResponse {
        code_id,
        wasm_byte_code,
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

pub fn wasm_raw(store: &dyn Storage, contract: &str, key: &[u8]) -> Result<WasmRawResponse> {
    let contract_addr = address::validate(contract)?;
    let backend_store = backend_store_read(store, &contract_addr);
    let (value, _) = backend_store.get(key);
    Ok(WasmRawResponse {
        value: value?.map(Binary),
    })
}

pub fn wasm_smart(store: &dyn Storage, contract: &str, msg: &[u8]) -> Result<WasmSmartResponse> {
    let contract_addr = address::validate(contract)?;
    let backend_store = backend_store_read(store, &contract_addr);

    let code_id = match ACCOUNTS.may_load(store, &contract_addr)? {
        Some(Account::Contract {
            code_id,
            ..
        }) => code_id,
        Some(Account::Base {
            ..
        }) => return Err(Error::account_is_not_contract(contract_addr)),
        None => return Err(Error::account_not_found(contract_addr)),
    };
    let code = CODES.load(store, code_id)?;

    let mut instance = Instance::from_code(
        &code,
        Backend {
            api: BackendApi,
            storage: backend_store,
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
