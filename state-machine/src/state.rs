use cosmwasm_std::{Addr, Binary, Storage};
use cw_sdk::Account;
use cw_storage_plus::{Item, Map};

use crate::error::{Error, Result};

/// The chain's identifier. Used to prevent tx replay attacks.
/// Must be in the format `{name}-{revision_number}` per IBC specs.
pub const CHAIN_ID: Item<String> = Item::new("chain_id");

/// Height of the last committed block.
///
/// It makes more sense to use u64 rather than i64, as block heights shouldn't
/// be negative. However Tendermint ABCI uses i64 for some reason, so we just
/// follow their standard.
pub const BLOCK_HEIGHT: Item<i64> = Item::new("block_height");

/// Time of the last committed block.
pub const BLOCK_TIME: Item<u64> = Item::new("block_time");

/// Height of the current pending block.
/// Set to (last committed height + 1) during the "BeginBlock" ABCI request.
pub const PENDING_HEIGHT: Item<Option<i64>> = Item::new("pending_height");

/// Time of the current pending block.
/// Updated by the "BeginBlock" ABCI request.
pub const PENDING_TIME: Item<Option<u64>> = Item::new("pending_time");

/// The total number of wasm byte codes stored on chain.
pub const CODE_COUNT: Item<u64> = Item::new("code_count");

/// The total number of contracts that have been instantiated.
pub const CONTRACT_COUNT: Item<u64> = Item::new("contract_count");

/// The wasm byte codes, indexed by code ids.
pub const CODES: Map<u64, Binary> = Map::new("codes");

/// Accounts, either base (i.e. externally-owned) accounts or smart contract
/// accounts, indexed by addresses.
pub const ACCOUNTS: Map<&Addr, Account<Addr>> = Map::new("accounts");

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
