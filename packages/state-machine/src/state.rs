use cosmwasm_std::{Addr, Binary, Storage, BlockInfo};
use cw_sdk::{indexes::AccountIndexes, Account};
use cw_storage_plus::{IndexedMap, Item, Map};

use crate::error::{Error, Result};

/// Info of the last committed block.
pub const BLOCK: Item<BlockInfo> = Item::new("block");

/// The total number of wasm byte codes stored on chain.
pub const CODE_COUNT: Item<u64> = Item::new("code_count");

/// The wasm byte codes, indexed by code ids.
pub const CODES: Map<u64, Binary> = Map::new("codes");

/// Accounts, either base (i.e. externally-owned) accounts or smart contract
/// accounts, indexed by addresses.
/// Contracts are additionally indexed by their labels, which must be unique.
pub const ACCOUNTS: IndexedMap<&Addr, Account<Addr>, AccountIndexes> = IndexedMap::new(
    "accounts",
    AccountIndexes::new("accounts__label"),
);

/// Helper function for loading the wasm code of a given contract address.
pub fn code_by_address(store: &dyn Storage, contract_addr: &Addr) -> Result<Binary> {
    let code_id = match ACCOUNTS.may_load(store, contract_addr)? {
        Some(Account::Contract {
            code_id,
            ..
        }) => code_id,
        Some(Account::Base {
            ..
        }) => {
            return Err(Error::account_is_not_contract(contract_addr));
        },
        None => {
            return Err(Error::account_not_found(contract_addr));
        },
    };
    CODES.load(store, code_id).map_err(Error::from)
}
