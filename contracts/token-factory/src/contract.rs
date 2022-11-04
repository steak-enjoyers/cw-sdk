use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{
    error::ContractError,
    execute,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UpdateTokenMsg},
    query,
};

pub const CONTRACT_NAME: &str = "crates.io:cw-token-factory";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    execute::init(deps, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateFee {
            token_creation_fee,
        } => execute::update_fee(deps, info, token_creation_fee),
        ExecuteMsg::WithdrawFee {
            to,
        } => execute::withdraw_fee(deps, env, info, to),
        ExecuteMsg::CreateToken {
            nonce,
            admin,
            after_send_hook,
        } => execute::create_token(deps, info, nonce, admin, after_send_hook),
        ExecuteMsg::UpdateToken(UpdateTokenMsg {
            denom,
            admin,
            after_send_hook,
        }) => execute::update_token(deps, info, denom, admin),
        ExecuteMsg::Mint {
            to,
            denom,
            amount,
        } => execute::mint(deps, info, to, denom, amount),
        ExecuteMsg::Burn {
            from,
            denom,
            amount,
        } => execute::burn(deps, info, from, denom, amount),
        ExecuteMsg::ForceTransfer {
            from,
            to,
            denom,
            amount,
        } => execute::force_transfer(deps, info, from, to, denom, amount),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Token {
            denom,
        } => to_binary(&query::token(deps, denom)?),
        QueryMsg::Tokens {
            start_after,
            limit,
        } => to_binary(&query::tokens(deps, start_after, limit)?),
    }
}
