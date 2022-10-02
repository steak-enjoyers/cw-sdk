use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

/// Tracks the total supply of denoms
pub const SUPPLIES: Map<&str, Uint128> = Map::new("supplies");

/// Tracks the account balance of each denom
pub const BALANCES: Map<(&Addr, &str), Uint128> = Map::new("balances");
