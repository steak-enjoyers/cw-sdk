use cosmwasm_std::{Coin, Deps, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

use crate::{
    msg::{Config, Minter},
    state::{BALANCES, CONFIG, MINTER_NAMESPACES, SUPPLIES},
};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub fn config(deps: Deps) -> StdResult<Config<String>> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(Config {
        owner: cfg.owner.into(),
    })
}

pub fn minter(deps: Deps, minter: String) -> StdResult<Minter> {
    let minter_addr = deps.api.addr_validate(&minter)?;
    let namespaces = MINTER_NAMESPACES.may_load(deps.storage, &minter_addr)?.unwrap_or_default();
    Ok(Minter {
        address: minter,
        namespaces,
    })
}

pub fn minters(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Minter>> {
    // we skip the address validation because not necessary
    let start = start_after.map(|minter| Bound::ExclusiveRaw(minter.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    MINTER_NAMESPACES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (address, namespaces) = item?;
            Ok(Minter {
                address: address.into(),
                namespaces,
            })
        })
        .collect()
}

pub fn supply(deps: Deps, denom: String) -> StdResult<Coin> {
    let amount = SUPPLIES.may_load(deps.storage, &denom)?.unwrap_or_else(Uint128::zero);
    Ok(Coin {
        denom,
        amount,
    })
}

pub fn supplies(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Coin>> {
    let start = start_after.map(|denom| Bound::ExclusiveRaw(denom.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    SUPPLIES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (denom, amount) = item?;
            Ok(Coin {
                denom,
                amount,
            })
        })
        .collect()
}

pub fn balance(deps: Deps, address: String, denom: String) -> StdResult<Coin> {
    let addr = deps.api.addr_validate(&address)?;
    let amount = BALANCES.may_load(deps.storage, (&addr, &denom))?.unwrap_or_else(Uint128::zero);
    Ok(Coin {
        denom,
        amount,
    })
}

pub fn balances(
    deps: Deps,
    address: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Coin>> {
    let addr = deps.api.addr_validate(&address)?;

    let start = start_after.map(|denom| Bound::ExclusiveRaw(denom.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    BALANCES
        .prefix(&addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (denom, amount) = item?;
            Ok(Coin {
                denom,
                amount,
            })
        })
        .collect()
}
