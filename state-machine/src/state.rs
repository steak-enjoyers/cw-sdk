use cosmwasm_std::{Addr, Binary};
use cw_sdk::Account;
use cw_storage_plus::{Item, Map};

pub const CHAIN_ID: Item<String> = Item::new("chain_id");

pub const HEIGHT: Item<i64> = Item::new("height");

pub const CODE_COUNT: Item<u64> = Item::new("code_count");

pub const CONTRACT_COUNT: Item<u64> = Item::new("contract_count");

pub const CODES: Map<u64, Binary> = Map::new("codes");

pub const ACCOUNTS: Map<&Addr, Account<Addr>> = Map::new("accounts");
