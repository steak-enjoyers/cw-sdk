use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::msg::{Config, TokenConfig};

/// The contract's configuration
pub const CONFIG: Item<Config<Addr>> = Item::new("config");

/// Configuration of tokens indexed by creator address and subdenom
pub const TOKEN_CONFIGS: Map<(&Addr, &str), TokenConfig> = Map::new("tkn_cfgs");
