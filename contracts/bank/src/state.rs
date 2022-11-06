use cosmwasm_std::{Addr, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

use crate::{
    denom::{Denom, Namespace, NamespaceConfig},
    msg::Config,
};

pub const CONFIG: Item<Config<Addr>> = Item::new("config");
pub const NAMESPACE_CONFIGS: Map<&Namespace, NamespaceConfig> = Map::new("ns_cfgs");
pub const SUPPLIES: Map<&Denom, Uint128> = Map::new("supplies");
pub const BALANCES: Map<(&Addr, &Denom), Uint128> = Map::new("balances");

/// Increase the total supply of a denom by the specified amount.
pub fn increase_supply(store: &mut dyn Storage, denom: &Denom, amount: Uint128) -> StdResult<()> {
    SUPPLIES.update(store, denom, |opt| {
        opt.unwrap_or_else(Uint128::zero).checked_add(amount).map_err(StdError::from)
    })?;
    Ok(())
}

/// Decrease the total supply of a denom by the specified amount.
/// Delete the record from contract storage if reduced to zero.
pub fn decrease_supply(store: &mut dyn Storage, denom: &Denom, amount: Uint128) -> StdResult<()> {
    let supply = SUPPLIES
        .may_load(store, denom)?
        .unwrap_or_else(Uint128::zero)
        .checked_sub(amount)?;

    if supply.is_zero() {
        SUPPLIES.remove(store, denom);
    } else {
        SUPPLIES.save(store, denom, &supply)?;
    }

    Ok(())
}

/// Increase an account's balance of a denom by the specified amount.
pub fn increase_balance(
    store: &mut dyn Storage,
    addr: &Addr,
    denom: &Denom,
    amount: Uint128,
) -> StdResult<()> {
    BALANCES.update(store, (addr, denom), |opt| {
        opt.unwrap_or_else(Uint128::zero).checked_add(amount).map_err(StdError::from)
    })?;
    Ok(())
}

/// Decrease an account's balance of a denom by the specified amount.
/// Delete the record from contract storage if reduced to zero.
pub fn decrease_balance(
    store: &mut dyn Storage,
    addr: &Addr,
    denom: &Denom,
    amount: Uint128,
) -> StdResult<()> {
    let balance = BALANCES
        .may_load(store, (addr, denom))?
        .unwrap_or_else(Uint128::zero)
        .checked_sub(amount)?;

    if balance.is_zero() {
        BALANCES.remove(store, (addr, denom));
    } else {
        BALANCES.save(store, (addr, denom), &balance)?;
    }

    Ok(())
}
