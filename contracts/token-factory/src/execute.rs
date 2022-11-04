use cosmwasm_std::{to_binary, Coin, DepsMut, Env, MessageInfo, Response, WasmMsg};

use cw_bank::msg as bank;
use cw_sdk::helpers::{stringify_coins, stringify_option};

use crate::{error::ContractError, msg::Config, state::CONFIG};

pub fn init(deps: DepsMut, cfg: Config<String>) -> Result<Response, ContractError> {
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&cfg.owner)?,
            bank: deps.api.addr_validate(&cfg.bank)?,
            token_creation_fee: cfg.token_creation_fee,
        },
    )?;

    Ok(Response::default())
}

pub fn update_fee(
    deps: DepsMut,
    info: MessageInfo,
    token_creation_fee: Option<Coin>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut cfg| {
        if info.sender != cfg.owner {
            return Err(ContractError::NotOwner);
        }

        cfg.token_creation_fee = token_creation_fee;
        Ok(cfg)
    })?;

    Ok(Response::new()
        .add_attribute("action", "token-factory/update_fee")
        .add_attribute("new_fee", stringify_option(token_creation_fee)))
}

pub fn withdraw_fee(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    if info.sender != cfg.owner {
        return Err(ContractError::NotOwner);
    }

    let coins: Vec<Coin> = deps.querier.query_wasm_smart(
        &cfg.bank,
        &bank::QueryMsg::Balances {
            address: env.contract.address.to_string(),
            start_after: None,
            limit: None,
        },
    )?;

    if coins.is_empty() {
        return Err(ContractError::NoBalance);
    }

    let to = to.unwrap_or_else(|| info.sender.into());

    Ok(Response::new()
        .add_attribute("action", "token-factory/withdraw_fee")
        .add_attribute("to", &to)
        .add_attribute("coins", stringify_coins(&coins))
        .add_message(WasmMsg::Execute {
            contract_addr: cfg.bank.into(),
            msg: to_binary(&bank::ExecuteMsg::Send {
                to,
                coins,
            })?,
            funds: vec![],
        }))
}
