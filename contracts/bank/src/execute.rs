use cosmwasm_std::{Addr, Coin, DepsMut, MessageInfo, Response, Uint128};

use crate::error::ContractError;
use crate::msg::Balance;
use crate::state::{BALANCES, SUPPLIES};

pub fn init(deps: DepsMut, balances: Vec<Balance>) -> Result<Response, ContractError> {
    for balance in &balances {
        let addr = deps.api.addr_validate(&balance.address)?;

        for coin in &balance.coins {
            SUPPLIES.update(deps.storage, &coin.denom, |opt| {
                opt
                    .unwrap_or_else(Uint128::zero)
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

pub fn mint(
    deps: DepsMut,
    info: MessageInfo,
    to: String,
    amount: Coin,
) -> Result<Response, ContractError> {
    // TODO: currently anyone can mint. need to implement a minter whitelist or a namespacing
    // mechanism similar to x/tokenfactory's.

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
    coins
        .iter()
        .map(|coin| coin.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
