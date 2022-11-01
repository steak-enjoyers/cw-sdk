use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use crate::{
    denom::{Denom, Namespace, NamespaceConfig},
    msg::Config,
};

pub const CONFIG: Item<Config<Addr>> = Item::new("config");
pub const NAMESPACE_CONFIGS: Map<&Namespace, NamespaceConfig> = Map::new("ns_cfgs");
pub const SUPPLIES: Map<&Denom, Uint128> = Map::new("supplies");
pub const BALANCES: Map<(&Addr, &Denom), Uint128> = Map::new("balances");
