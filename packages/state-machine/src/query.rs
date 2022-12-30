use cosmwasm_std::{testing::mock_env, Binary, Order, Storage};
use cosmwasm_vm::{call_query, Backend, Instance, InstanceOptions, Storage as VmStorage};
use cw_storage_plus::Bound;

use cw_sdk::{
    address,
    paginate::{collect, paginate_indexed_map, paginate_map},
    Account, AccountResponse, CodeResponse, ContractResponse, InfoResponse, WasmRawResponse,
    WasmSmartResponse,
};

use crate::{
    backend::{BackendApi, BackendQuerier, ContractSubstore},
    error::Result,
    state::{code_by_address, ACCOUNTS, BLOCK, CODES, CODE_COUNT},
};

pub fn info(store: &dyn Storage) -> Result<InfoResponse> {
    Ok(InfoResponse {
        code_count: CODE_COUNT.load(store)?,
        last_committed_block: BLOCK.load(store)?,
    })
}

pub fn account(store: &dyn Storage, address: String) -> Result<AccountResponse> {
    let addr = address::validate(&address)?;
    let account = ACCOUNTS.load(store, &addr)?;
    Ok(AccountResponse {
        address,
        account: account.into(),
    })
}

pub fn accounts(
    store: &dyn Storage,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<AccountResponse>> {
    let start = start_after.map(|address| Bound::ExclusiveRaw(address.into_bytes()));
    paginate_indexed_map(ACCOUNTS, store, start, limit, |address, account| {
        Ok(AccountResponse {
            address: address.into(),
            account: account.into(),
        })
    })
}

pub fn contract(store: &dyn Storage, label: String) -> Result<ContractResponse> {
    let (address, account) = ACCOUNTS.idx.label.load(store, label)?;
    match account {
        Account::Contract {
            code_id,
            label,
            admin,
        } => Ok(ContractResponse {
            address: address.into(),
            code_id,
            label,
            admin: admin.map(String::from),
        }),
        _ => unreachable!(),
    }
}

pub fn contracts(
    store: &dyn Storage,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ContractResponse>> {
    let start = start_after.map(Bound::exclusive);
    let iter = ACCOUNTS.idx.label.range(store, start, None, Order::Ascending);
    collect(iter, limit, |address, account| match account {
        Account::Contract {
            code_id,
            label,
            admin,
        } => Ok(ContractResponse {
            address: address.into(),
            code_id,
            label,
            admin: admin.map(String::from),
        }),
        _ => unreachable!(),
    })
}

pub fn code(store: &dyn Storage, code_id: u64) -> Result<CodeResponse> {
    Ok(CodeResponse {
        code_id,
        wasm_byte_code: CODES.load(store, code_id)?,
    })
}

pub fn codes(
    store: &dyn Storage,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> Result<Vec<CodeResponse>> {
    let start = start_after.map(Bound::exclusive);
    paginate_map(CODES, store, start, limit, |code_id, wasm_byte_code| {
        Ok(CodeResponse {
            code_id,
            wasm_byte_code,
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

pub fn wasm_smart(
    store: impl Storage + 'static,
    contract: &str,
    msg: &[u8],
) -> Result<WasmSmartResponse> {
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