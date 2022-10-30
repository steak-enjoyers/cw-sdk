use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{
    error::ContractError,
    execute,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
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
) -> StdResult<Response> {
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
        ExecuteMsg::SetFee {
            token_creation_fee,
        } => execute::set_fee(deps, info, token_creation_fee),
        ExecuteMsg::CreateToken {
            subdenom,
            admin,
            before_transfer_hook,
        } => execute::create_token(deps, info, subdenom, admin, before_transfer_hook),
        ExecuteMsg::SetAdmin {
            denom,
            admin,
        } => execute::set_admin(deps, info, denom, admin),
        ExecuteMsg::SetBeforeTransferHook {
            denom,
            before_transfer_hook,
        } => execute::set_before_transfer_hook(deps, info, denom, before_transfer_hook),
        ExecuteMsg::Mint {
            to,
            amount,
        } => execute::mint(deps, info, to, amount),
        ExecuteMsg::Burn {
            from,
            amount,
        } => execute::burn(deps, info, from, amount),
        ExecuteMsg::ForceTransfer {
            from,
            to,
            amount,
        } => execute::force_transfer(deps, info, from, to, amount),
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
