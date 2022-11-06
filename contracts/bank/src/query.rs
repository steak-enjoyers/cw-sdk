use std::str::FromStr;

use cosmwasm_std::{Coin, Deps, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

use crate::{
    denom::{Denom, Namespace},
    error::ContractError,
    msg::{Config, NamespaceResponse},
    state::{BALANCES, CONFIG, NAMESPACE_CONFIGS, SUPPLIES},
};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

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
    let start = start_after
        .map(|s| Namespace::from_str(&s))
        .transpose()?
        .map(|ns| Bound::ExclusiveRaw(ns.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    NAMESPACE_CONFIGS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (namespace, cfg) = item?;
            Ok(NamespaceResponse {
                namespace: namespace.into(),
                admin: cfg.admin.map(String::from),
                after_transfer_hook: cfg.after_transfer_hook.map(String::from),
            })
        })
        .collect()
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
    let start = start_after
        .map(|s| Denom::from_str(&s))
        .transpose()?
        .map(|d| Bound::ExclusiveRaw(d.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    SUPPLIES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (denom, amount) = item?;
            Ok(Coin {
                denom: denom.into(),
                amount,
            })
        })
        .collect()
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
    let addr = deps.api.addr_validate(&address)?;
    let start = start_after
        .map(|s| Denom::from_str(&s))
        .transpose()?
        .map(|d| Bound::ExclusiveRaw(d.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    BALANCES
        .prefix(&addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (denom, amount) = item?;
            Ok(Coin {
                denom: denom.into(),
                amount,
            })
        })
        .collect()
}
