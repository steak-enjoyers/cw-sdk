use std::str::FromStr;

use cosmwasm_std::{
    to_binary, Addr, Api, Coin, DepsMut, MessageInfo, Response, StdResult, Storage, Uint128,
    WasmMsg,
};

use crate::{
    denom::{Denom, Namespace, NamespaceAdminExecuteMsg, NamespaceConfig},
    error::ContractError,
    msg::{Balance, Config, UpdateNamespaceMsg},
    state::{
        decrease_balance, decrease_supply, increase_balance, increase_supply, BALANCES, CONFIG,
        NAMESPACE_CONFIGS,
    },
};

pub fn init(
    deps: DepsMut,
    owner: String,
    balances: Vec<Balance>,
    namespace_cfgs: Vec<UpdateNamespaceMsg>,
) -> Result<Response, ContractError> {
    // 1. Initialize config
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&owner)?,
        },
    )?;

    // 2. Initialize balances
    // NOTE: Must ensure that for each address, there is no duplication in coin denoms, and coin
    // amount is non-zero.
    for Balance {
        address,
        coins,
    } in balances
    {
        let addr = deps.api.addr_validate(&address)?;

        for coin in coins {
            if coin.amount.is_zero() {
                return Err(ContractError::zero_init_balance(address, coin.denom));
            }

            let denom = Denom::from_str(&coin.denom)?;

            increase_supply(deps.storage, &denom, coin.amount)?;

            BALANCES.update(deps.storage, (&addr, &denom), |balance| {
                if balance.is_none() {
                    Ok(coin.amount)
                } else {
                    Err(ContractError::duplicate_balance(&addr, denom.clone()))
                }
            })?;
        }
    }

    // 2. Initialize namespaces
    // NOTE: Must ensure that for each namespace, there is only one admin. However, an admin can
    // administer multiple namespaces.
    for UpdateNamespaceMsg {
        namespace,
        admin,
        after_send_hook,
    } in namespace_cfgs
    {
        let ns = Namespace::from_str(&namespace)?;

        NAMESPACE_CONFIGS.update(deps.storage, &ns, |namespace_cfg| {
            if namespace_cfg.is_none() {
                Ok(NamespaceConfig {
                    admin: validate_optional_addr(deps.api, admin.as_ref())?,
                    after_send_hook: validate_optional_addr(deps.api, after_send_hook.as_ref())?,
                })
            } else {
                Err(ContractError::duplicate_namespace(ns.clone()))
            }
        })?;
    }

    Ok(Response::default())
}

pub fn update_namespace(
    deps: DepsMut,
    info: MessageInfo,
    namespace: String,
    admin: Option<String>,
    after_send_hook: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let ns = Namespace::from_str(&namespace)?;

    // The sender must be either the contract owner or the namespace's admin
    if info.sender != cfg.owner {
        assert_namespace_admin(deps.as_ref().storage, &ns, &info.sender)?;
    }

    NAMESPACE_CONFIGS.save(
        deps.storage,
        &ns,
        &NamespaceConfig {
            admin: validate_optional_addr(deps.api, admin.as_ref())?,
            after_send_hook: validate_optional_addr(deps.api, after_send_hook.as_ref())?,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "bank/update_namespace")
        .add_attribute("namespace", namespace)
        .add_attribute("admin", stringify_option(admin))
        .add_attribute("after_send_hook", stringify_option(after_send_hook)))
}

pub fn mint(
    deps: DepsMut,
    info: MessageInfo,
    to: String,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let d = Denom::from_str(&denom)?;
    let ns = (&d).into();
    let to_addr = deps.api.addr_validate(&to)?;

    assert_non_zero_amount(&denom, amount)?;
    assert_namespace_admin(deps.storage, &ns, &info.sender)?;

    increase_supply(deps.storage, &d, amount)?;
    increase_balance(deps.storage, &to_addr, &d, amount)?;

    Ok(Response::new()
        .add_attribute("action", "bank/mint")
        .add_attribute("minter", info.sender)
        .add_attribute("to", to)
        .add_attribute("coin", format!("{amount}{denom}")))
}

pub fn burn(
    deps: DepsMut,
    info: MessageInfo,
    from: String,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let d = Denom::from_str(&denom)?;
    let ns = (&d).into();
    let from_addr = deps.api.addr_validate(&from)?;

    assert_non_zero_amount(&denom, amount)?;
    assert_namespace_admin(deps.storage, &ns, &info.sender)?;

    decrease_supply(deps.storage, &d, amount)?;
    decrease_balance(deps.storage, &from_addr, &d, amount)?;

    Ok(Response::new()
        .add_attribute("action", "bank/burn")
        .add_attribute("burner", info.sender)
        .add_attribute("from", from)
        .add_attribute("coin", format!("{amount}{denom}")))
}

pub fn send(
    deps: DepsMut,
    info: MessageInfo,
    to: String,
    coins: Vec<Coin>,
) -> Result<Response, ContractError> {
    transfer(
        deps.storage,
        &info.sender,
        &deps.api.addr_validate(&to)?,
        &coins,
    )
}

pub fn sudo_transfer(
    deps: DepsMut,
    from: String,
    to: String,
    coins: Vec<Coin>,
) -> Result<Response, ContractError> {
    transfer(
        deps.storage,
        &deps.api.addr_validate(&from)?,
        &deps.api.addr_validate(&to)?,
        &coins,
    )
}

pub fn force_transfer(
    deps: DepsMut,
    from: String,
    to: String,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    transfer(
        deps.storage,
        &deps.api.addr_validate(&from)?,
        &deps.api.addr_validate(&to)?,
        &[Coin {
            denom,
            amount,
        }],
    )
}

/// Internal method: perform transfers of multiple coins.
/// For each coin,
/// 1. Update balances
/// 2. If `after_send_hook` is defined for its namespace, compose a message to invoke the hook
fn transfer(
    store: &mut dyn Storage,
    from_addr: &Addr,
    to_addr: &Addr,
    coins: &[Coin],
) -> Result<Response, ContractError> {
    let mut msgs = vec![];

    for coin in coins {
        let d = Denom::from_str(&coin.denom)?;
        let ns = (&d).into();

        assert_non_zero_amount(&coin.denom, coin.amount)?;

        decrease_balance(store, from_addr, &d, coin.amount)?;
        increase_balance(store, to_addr, &d, coin.amount)?;

        if let Some(namespace_cfg) = NAMESPACE_CONFIGS.may_load(store, &ns)? {
            if let Some(after_send_hook) = namespace_cfg.after_send_hook {
                msgs.push(WasmMsg::Execute {
                    contract_addr: after_send_hook.into(),
                    msg: to_binary(&NamespaceAdminExecuteMsg::AfterTransfer {
                        from: from_addr.to_string(),
                        to: to_addr.to_string(),
                        coin: coin.clone(),
                    })?,
                    funds: vec![],
                });
            }
        }
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "bank/transfer")
        .add_attribute("from", from_addr)
        .add_attribute("to", to_addr)
        .add_attribute("coins", stringify_coins(coins)))
}

fn assert_non_zero_amount(denom: &str, amount: Uint128) -> Result<(), ContractError> {
    if amount.is_zero() {
        Err(ContractError::zero_amount(denom))
    } else {
        Ok(())
    }
}

fn assert_namespace_admin(
    store: &dyn Storage,
    namespace: &Namespace,
    sender: &Addr,
) -> Result<(), ContractError> {
    if let Some(namespace_cfg) = NAMESPACE_CONFIGS.may_load(store, namespace)? {
        if let Some(admin) = namespace_cfg.admin {
            if *sender == admin {
                return Ok(());
            }
        }
        Err(ContractError::not_namespace_admin(namespace))
    } else {
        Err(ContractError::non_exist_namespace(namespace))
    }
}

fn stringify_coins(coins: &[Coin]) -> String {
    coins.iter().map(|coin| coin.to_string()).collect::<Vec<_>>().join(",")
}

fn stringify_option(opt: Option<impl ToString>) -> String {
    opt.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn validate_optional_addr(api: &dyn Api, opt: Option<&String>) -> StdResult<Option<Addr>> {
    opt.map(|s| api.addr_validate(s)).transpose()
}
