use std::collections::BTreeSet;

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Map, Item};

use crate::msg::Config;

/// The contract's configuration
pub const CONFIG: Item<Config<Addr>> = Item::new("config");

/// The authorized namespaces for each minter account
pub const MINTER_NAMESPACES: Map<&Addr, BTreeSet<Option<String>>> = Map::new("minters");

/// Tracks the total supply of denoms
pub const SUPPLIES: Map<&str, Uint128> = Map::new("supplies");

/// Tracks the account balance of each denom
pub const BALANCES: Map<(&Addr, &str), Uint128> = Map::new("balances");
