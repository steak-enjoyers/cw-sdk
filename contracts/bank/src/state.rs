use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use crate::{
    denom::{Namespace, NamespaceConfig},
    msg::Config,
};

/// The contract's configuration
pub const CONFIG: Item<Config<Addr>> = Item::new("config");

/// The authorized namespaces for each minter account
pub const NAMESPACE_CONFIGS: Map<&Namespace, NamespaceConfig> = Map::new("ns_cfgs");

/// Tracks the total supply of denoms
pub const SUPPLIES: Map<&str, Uint128> = Map::new("supplies");

/// Tracks the account balance of each denom
pub const BALANCES: Map<(&Addr, &str), Uint128> = Map::new("balances");
