use std::str::FromStr;

use cosmwasm_std::{Coin, Deps, StdResult, Uint128};
use cw_sdk::paginate::{paginate_map, paginate_map_prefix};
use cw_storage_plus::Bound;

use crate::{
    denom::{Denom, Namespace},
    error::ContractError,
    msg::{Config, NamespaceResponse},
    state::{BALANCES, CONFIG, NAMESPACE_CONFIGS, SUPPLIES},
};

pub fn config(deps: Deps) -> StdResult<Config<String>> {
    CONFIG.load(deps.storage).map(|cfg| Config {
        owner: cfg.owner.into(),
    })
}

pub fn namespace(deps: Deps, namespace: String) -> Result<NamespaceResponse, ContractError> {
    let ns = Namespace::from_str(&namespace)?;
    let cfg = NAMESPACE_CONFIGS.load(deps.storage, &ns)?;
    Ok(NamespaceResponse {
        namespace,
        admin: cfg.admin.map(String::from),
        after_transfer_hook: cfg.after_transfer_hook.map(String::from),
    })
}

pub fn namespaces(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<NamespaceResponse>, ContractError> {
    let start = start_after.map(|namespace| Bound::ExclusiveRaw(namespace.into_bytes()));
    paginate_map(NAMESPACE_CONFIGS, deps.storage, start, limit, |namespace, cfg| {
        Ok(NamespaceResponse {
            namespace: namespace.into(),
            admin: cfg.admin.map(String::from),
            after_transfer_hook: cfg.after_transfer_hook.map(String::from),
        })
    })
}

pub fn supply(deps: Deps, denom: String) -> Result<Coin, ContractError> {
    let d = Denom::from_str(&denom)?;
    let supply = SUPPLIES.may_load(deps.storage, &d)?;
    Ok(Coin {
        denom,
        amount: supply.unwrap_or_else(Uint128::zero),
    })
}

pub fn supplies(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Coin>, ContractError> {
    let start = start_after.map(|denom| Bound::ExclusiveRaw(denom.into_bytes()));
    paginate_map(SUPPLIES, deps.storage, start, limit, |denom, amount| {
        Ok(Coin {
            denom: denom.into(),
            amount,
        })
    })
}

pub fn balance(deps: Deps, address: String, denom: String) -> Result<Coin, ContractError> {
    let addr = deps.api.addr_validate(&address)?;
    let d = Denom::from_str(&denom)?;
    let balance = BALANCES.may_load(deps.storage, (&addr, &d))?;
    Ok(Coin {
        denom,
        amount: balance.unwrap_or_else(Uint128::zero),
    })
}

pub fn balances(
    deps: Deps,
    address: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Coin>, ContractError> {
    let start = start_after.map(|denom| Bound::ExclusiveRaw(denom.into_bytes()));
    let prefix = deps.api.addr_validate(&address)?;
    paginate_map_prefix(BALANCES, deps.storage, &prefix, start, limit, |denom, amount| {
        Ok(Coin {
            denom: denom.into(),
            amount,
        })
    })
}
