use cosmwasm_std::Addr;
use cw_storage_plus::{Map, Item};

use crate::types::{Token, Config};

/// The contract's configuration
pub const CONFIG: Item<Config<Addr>> = Item::new("config");

/// Tokens indexed by creator address and subdenom
pub const TOKENS: Map<(&Addr, &str), Token<Addr>> = Map::new("tokens");
