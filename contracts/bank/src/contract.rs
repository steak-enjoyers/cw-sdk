use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
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
    execute::init(deps, msg.balances)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {
            to,
            amount,
        } => execute::mint(deps, info, to, amount),
        ExecuteMsg::Send {
            to,
            amount,
        } => execute::send(deps, info.sender, to, amount),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
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
