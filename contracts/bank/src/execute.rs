use std::collections::BTreeSet;

use cosmwasm_std::{Addr, Coin, DepsMut, MessageInfo, Response, Uint128};

use crate::{
    error::ContractError,
    denom::{
        denom::{namespace_to_attr, validate_denom, validate_namespace, Denom, Namespace},
        dup::DupChecker,
    },
    msg::{Balance, Config, Minter},
    state::{BALANCES, CONFIG, MINTER_NAMESPACES, SUPPLIES},
};

pub fn init(
    deps: DepsMut,
    owner: String,
    minters: Vec<Minter>,
    balances: Vec<Balance>,
) -> Result<Response, ContractError> {
    // 1. initialize config
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&owner)?,
        },
    )?;

    // 2. initialize minting authorizations
    let mut dc = DupChecker::new("minter address");
    for Minter {
        address,
        namespaces,
    } in &minters
    {
        dc.assert_no_dup(address)?;
        let addr = deps.api.addr_validate(address)?;

        namespaces.iter().try_for_each(validate_namespace)?;

        MINTER_NAMESPACES.save(deps.storage, &addr, namespaces)?;
    }

    // 3. initialize coin balances
    let mut dc = DupChecker::new("balance address");
    for Balance {
        address,
        coins,
    } in &balances
    {
        dc.assert_no_dup(address)?;
        let addr = deps.api.addr_validate(address)?;

        for coin in coins {
            SUPPLIES.update(deps.storage, &coin.denom, |opt| {
                opt.unwrap_or_else(Uint128::zero)
                    .checked_add(coin.amount)
                    .map_err(ContractError::from)
            })?;
            BALANCES.update(deps.storage, (&addr, &coin.denom), |opt| {
                if opt.is_none() {
                    Ok(coin.amount)
                } else {
                    Err(ContractError::duplicate_denom(&coin.denom))
                }
            })?;
        }
    }

    Ok(Response::default())
}

pub fn set_minter(
    deps: DepsMut,
    info: MessageInfo,
    minter: String,
    namespaces: BTreeSet<Namespace>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.owner {
        return Err(ContractError::NotOwner);
    }

    let minter_addr = deps.api.addr_validate(&minter)?;
    namespaces.iter().try_for_each(validate_namespace)?;
    MINTER_NAMESPACES.save(deps.storage, &minter_addr, &namespaces)?;

    Ok(Response::new()
        .add_attribute("action", "bank/set_minter")
        .add_attribute("minter", minter)
        .add_attributes(namespaces.iter().map(namespace_to_attr)))
}

pub fn mint(
    deps: DepsMut,
    info: MessageInfo,
    to: String,
    amount: Coin,
) -> Result<Response, ContractError> {
    let Denom {
        namespace,
        ..
    } = validate_denom(&amount.denom)?;

    let namespaces = MINTER_NAMESPACES.may_load(deps.storage, &info.sender)?.unwrap_or_default();
    if !namespaces.contains(&namespace) {
        return Err(ContractError::not_minter(&namespace));
    }

    let to_addr = deps.api.addr_validate(&to)?;

    BALANCES.update(deps.storage, (&to_addr, &amount.denom), |opt| {
        opt
            .unwrap_or_else(Uint128::zero)
            .checked_add(amount.amount)
            .map_err(ContractError::from)
    })?;
    SUPPLIES.update(deps.storage, &amount.denom, |opt| {
        opt
            .unwrap_or_else(Uint128::zero)
            .checked_add(amount.amount)
            .map_err(ContractError::from)
    })?;

    Ok(Response::new()
        .add_attribute("action", "bank/mint")
        .add_attribute("minter", info.sender)
        .add_attribute("to", to)
        .add_attribute("amount", amount.to_string()))
}

pub fn send(
    deps: DepsMut,
    from_addr: Addr,
    to: String,
    amount: Vec<Coin>,
) -> Result<Response, ContractError> {
    let to_addr = deps.api.addr_validate(&to)?;

    for coin in &amount {
        BALANCES.update(deps.storage, (&from_addr, &coin.denom), |opt| {
            opt
                .unwrap_or_else(Uint128::zero)
                .checked_sub(coin.amount)
                .map_err(ContractError::from)
        })?;
        BALANCES.update(deps.storage, (&to_addr, &coin.denom), |opt| {
            opt
                .unwrap_or_else(Uint128::zero)
                .checked_add(coin.amount)
                .map_err(ContractError::from)
        })?;
    }

    Ok(Response::new()
        .add_attribute("action", "bank/send")
        .add_attribute("from", from_addr)
        .add_attribute("to", to)
        .add_attribute("amount", stringify_coins(&amount)))
}

pub fn stringify_coins(coins: &[Coin]) -> String {
    coins.iter().map(|coin| coin.to_string()).collect::<Vec<_>>().join(",")
}
