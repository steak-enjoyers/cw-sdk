use cosmwasm_std::{Addr, Binary, ContractResult, Env, Event, MessageInfo, Response, Storage, BlockInfo, TransactionInfo, ContractInfo};
use cosmwasm_vm::{call_instantiate, Backend, Instance, InstanceOptions, call_execute};

use cw_sdk::{address, hash::sha256, Account};
use cw_store::Cached;

use crate::{
    backend::{BackendApi, BackendQuerier, ContractSubstore},
    error::{Error, Result},
    state::{ACCOUNTS, CODES, CODE_COUNT, CONTRACT_COUNT, code_by_address},
    AddressGenerator,
};

pub fn store_code(
    store: &mut dyn Storage,
    sender_addr: &Addr,
    wasm_byte_code: &Binary,
) -> Result<Event> {
    // increment the code count
    let code_id = CODE_COUNT.update(store, |count| -> Result<_> {
        Ok(count + 1)
    })?;

    // save code to the store
    CODES.save(store, code_id, wasm_byte_code)?;

    Ok(Event::new("store_code")
        .add_attribute("sender", sender_addr)
        .add_attribute("code_id", code_id.to_string())
        .add_attribute("code_hash", hex::encode(sha256(wasm_byte_code))))
}

#[allow(clippy::too_many_arguments)]
pub fn instantiate_contract(
    store: impl Storage + 'static,
    block: BlockInfo,
    transaction: Option<TransactionInfo>,
    info: &MessageInfo,
    code_id: u64,
    msg: &[u8],
    label: String,
    admin: Option<Addr>,
    address_generator: AddressGenerator,
) -> Result<ContractResult<Response>> {
    let mut cache = Cached::new(store);

    // update contract count
    let instance_id = CONTRACT_COUNT.update(&mut cache, |count| -> Result<_> {
        Ok(count + 1)
    })?;

    // derive contract address
    // TODO: this match block is better move into cw_sdk::address
    let contract_addr = match address_generator {
        AddressGenerator::ByLabel => address::derive_from_label(&label)?,
        AddressGenerator::ByIds => address::derive_from_ids(code_id, instance_id)?,
    };

    let env = Env {
        block,
        transaction,
        contract: ContractInfo {
            // TODO: recycle the address later instead of cloning
            address: contract_addr.clone(),
        },
    };

    // load wasm binary code
    let code = CODES.load(&cache, code_id)?;

    // create the wasm instance and call the instantiate entry point
    let mut instance = Instance::from_code(
        &code,
        Backend {
            api: BackendApi,
            storage: ContractSubstore::new(cache, &contract_addr),
            querier: BackendQuerier,
        },
        InstanceOptions {
            gas_limit: u64::MAX,
            print_debug: true,
        },
        None,
    )?;
    let result = call_instantiate(&mut instance, &env, info, msg)?;

    // contract execution is finished; we recycle the cached store
    let mut cache = instance
        .recycle()
        .expect("[cw-state-machine]: failed to recycle instance")
        .storage
        .recycle();

    // if the contract execution is successful, we flush the state changes
    // occurred during the instantiation call to the underlying store, and save
    // the contract account.
    //
    // NOTE: do not save the account if one of the same address already exists.
    if result.is_ok() {
        cache.flush();
        let mut store = cache.recycle();
        ACCOUNTS.update(&mut store, &contract_addr, |opt| {
            if opt.is_some() {
                return Err(Error::account_found(&contract_addr));
            }
            Ok(Account::Contract {
                code_id,
                label,
                admin,
            })
        })?;
    }

    Ok(result)
}

pub fn execute_contract(
    store: impl Storage + 'static,
    env: &Env,
    info: &MessageInfo,
    msg: &[u8],
) -> Result<ContractResult<Response>> {
    let cache = Cached::new(store);

    // load wasm binary code
    let code = code_by_address(&cache, &env.contract.address)?;

    // create the wasm instance and call the execute entry point
    let mut instance = Instance::from_code(
        &code,
        Backend {
            api: BackendApi,
            storage: ContractSubstore::new(cache, &env.contract.address),
            querier: BackendQuerier,
        },
        InstanceOptions {
            gas_limit: u64::MAX,
            print_debug: true,
        },
        None,
    )?;
    let result = call_execute(&mut instance, env, info, msg)?;

    // contract execution is finished; we recycle the cached store
    let mut cache = instance
        .recycle()
        .expect("[cw-state-machine]: failed to recycle instance")
        .storage
        .recycle();

    // if the execution is successful, flush the state changes to the underlying store
    if result.is_ok() {
        cache.flush();
    }

    Ok(result)
}

pub fn migrate_contract(
    _store: impl Storage + 'static,
    _env: &Env,
    _code_id: u64,
    _msg: &[u8]
) -> Result<ContractResult<Response>> {
    todo!();
}
