use cosmwasm_std::{Addr, Binary, Storage};
use cw_sdk::Account;
use cw_storage_plus::{Item, Map};

use crate::error::{Error, Result};

pub const CHAIN_ID: Item<String> = Item::new("chain_id");

pub const HEIGHT: Item<i64> = Item::new("height");

pub const CODE_COUNT: Item<u64> = Item::new("code_count");

pub const CONTRACT_COUNT: Item<u64> = Item::new("contract_count");

pub const CODES: Map<u64, Binary> = Map::new("codes");

pub const ACCOUNTS: Map<&Addr, Account<Addr>> = Map::new("accounts");

/// Helper function for loading the wasm binary code of a given code id.
pub fn load_code_id(store: &dyn Storage, contract_addr: &Addr) -> Result<Binary> {
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
    CODES.load(store, code_id).map_err(Error::from)
}
