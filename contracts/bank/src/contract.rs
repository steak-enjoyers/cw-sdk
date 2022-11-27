#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use crate::{
    error::ContractError,
    execute,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg, UpdateNamespaceMsg},
    query,
};

pub const CONTRACT_NAME: &str = "crates.io:cw-bank";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    execute::init(deps, msg.owner, msg.balances, msg.namespace_cfgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::Transfer {
            from,
            to,
            coins,
        } => execute::sudo_transfer(deps, from, to, coins),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateNamespace(UpdateNamespaceMsg {
            namespace,
            admin,
            after_transfer_hook,
        }) => execute::update_namespace(deps, info, namespace, admin, after_transfer_hook),
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
        } => execute::burn(deps, info, from, denom, amount),
        ExecuteMsg::ForceTransfer {
            from,
            to,
            denom,
            amount,
        } => execute::force_transfer(deps, from, to, denom, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
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
    .map_err(ContractError::from)
}
