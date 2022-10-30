use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::{execute, query};

pub const CONTRACT_NAME: &str = "crates.io:cw-bank";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    execute::init(deps, msg.owner, msg.minters, msg.balances)
}

#[entry_point]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::Transfer {
            from,
            to,
            coins,
        } => execute::sudo_transfer(deps, from, to, coins),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateNamespace {
            namespace,
            admin,
            hookable,
        } => execute::set_minter(deps, info, namespace, admin, hookable),
        ExecuteMsg::Send {
            to,
            coins,
        } => execute::send(deps, info, to, coins),
        ExecuteMsg::Mint {
            to,
            denom,
            amount,
        } => execute::mint(deps, info, to, denom, amount),
        ExecuteMsg::Burn {
            from,
            denom,
            amount,
        } => execute::send(deps, info, from, denom, amount),
        ExecuteMsg::ForceTransfer {
            from,
            to,
            denom,
            amount,
        } => execute::force_transfer(deps, info, from, to, denom, amount),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Namespace {
            namespace,
        } => to_binary(&query::namespace(deps, namespace)?),
        QueryMsg::Namespaces {
            start_after,
            limit,
        } => to_binary(&query::namespaces(deps, start_after, limit)?),
        QueryMsg::Supply {
            denom,
        } => to_binary(&query::supply(deps, denom)?),
        QueryMsg::Supplies {
            start_after,
            limit,
        } => to_binary(&query::supplies(deps, start_after, limit)?),
        QueryMsg::Balance {
            address,
            denom,
        } => to_binary(&query::balance(deps, address, denom)?),
        QueryMsg::Balances {
            address,
            start_after,
            limit,
        } => to_binary(&query::balances(deps, address, start_after, limit)?),
    }
}
