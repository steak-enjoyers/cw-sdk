use cosmwasm_std::{
    to_binary, Addr, Api, Coin, DepsMut, MessageInfo, Response, StdResult, Storage, Uint128,
    WasmMsg,
};

use crate::{
    denom::{validate_denom, Namespace, NamespaceAdminExecuteMsg, NamespaceConfig},
    error::ContractError,
    msg::{Balance, Config, UpdateNamespaceMsg},
    state::{BALANCES, CONFIG, NAMESPACE_CONFIGS, SUPPLIES},
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
    // NOTE: Must ensure that for each address, there is no duplication in coin denoms.
    for Balance {
        address,
        coins,
    } in balances
    {
        let addr = deps.api.addr_validate(&address)?;

        for coin in coins {
            SUPPLIES.update(deps.storage, &coin.denom, |supply| {
                supply
                    .unwrap_or_else(Uint128::zero)
                    .checked_add(coin.amount)
                    .map_err(ContractError::from)
            })?;
            BALANCES.update(deps.storage, (&addr, &coin.denom), |balance| {
                if balance.is_none() {
                    Ok(coin.amount)
                } else {
                    Err(ContractError::duplicate_denom(&coin.denom))
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
        namespace.validate()?;

        NAMESPACE_CONFIGS.update(deps.storage, &namespace, |namespace_cfg| {
            if namespace_cfg.is_none() {
                Ok(NamespaceConfig {
                    admin: validate_optional_addr(deps.api, admin.as_ref())?,
                    after_send_hook: validate_optional_addr(deps.api, after_send_hook.as_ref())?,
                })
            } else {
                Err(ContractError::duplicate_namespace(&namespace))
            }
        })?;
    }

    Ok(Response::default())
}

pub fn update_namespace(
    deps: DepsMut,
    info: MessageInfo,
    namespace: Namespace,
    admin: Option<String>,
    after_send_hook: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // The sender must be either the contract owner or the namespace's admin
    if info.sender != cfg.owner {
        assert_namespace_admin(deps.as_ref().storage, &namespace, &info.sender)?;
    }

    namespace.validate()?;

    NAMESPACE_CONFIGS.save(
        deps.storage,
        &namespace,
        &NamespaceConfig {
            admin: validate_optional_addr(deps.api, admin.as_ref())?,
            after_send_hook: validate_optional_addr(deps.api, after_send_hook.as_ref())?,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "bank/update_namespace")
        .add_attribute("namespace", &namespace)
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
    let namespace = Namespace::extract_from_denom(&denom)?;

    assert_namespace_admin(deps.as_ref().storage, &namespace, &info.sender)?;

    let to_addr = deps.api.addr_validate(&to)?;

    // NOTE: We only need to validate the denom if this is the first time this denom is minted
    BALANCES.update(deps.storage, (&to_addr, &denom), |opt| {
        opt
            .unwrap_or_else(Uint128::zero)
            .checked_add(amount)
            .map_err(ContractError::from)
    })?;
    SUPPLIES.update(deps.storage, &denom, |opt| {
        if opt.is_none() {
            validate_denom(&denom)?;
        }
        opt
            .unwrap_or_else(Uint128::zero)
            .checked_add(amount)
            .map_err(ContractError::from)
    })?;

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
    let namespace = Namespace::extract_from_denom(&denom)?;

    assert_namespace_admin(deps.as_ref().storage, &namespace, &info.sender)?;

    let from_addr = deps.api.addr_validate(&from)?;

    BALANCES.update(deps.storage, (&from_addr, &denom), |opt| {
        opt
            .unwrap_or_else(Uint128::zero)
            .checked_sub(amount)
            .map_err(ContractError::from)
    })?;
    SUPPLIES.update(deps.storage, &denom, |opt| {
        opt
            .unwrap_or_else(Uint128::zero)
            .checked_sub(amount)
            .map_err(ContractError::from)
    })?;

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
    let to_addr = deps.api.addr_validate(&to)?;
    let msgs = transfer(deps.storage, &info.sender, &to_addr, &coins)?;
    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "bank/send")
        .add_attribute("from", info.sender)
        .add_attribute("to", to)
        .add_attribute("coins", stringify_coins(&coins)))
}

pub fn sudo_transfer(
    deps: DepsMut,
    from: String,
    to: String,
    coins: Vec<Coin>,
) -> Result<Response, ContractError> {
    let from_addr = deps.api.addr_validate(&from)?;
    let to_addr = deps.api.addr_validate(&to)?;
    let msgs = transfer(deps.storage, &from_addr, &to_addr, &coins)?;
    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "bank/sudo_transfer")
        .add_attribute("from", from)
        .add_attribute("to", to)
        .add_attribute("coins", stringify_coins(&coins)))
}

pub fn force_transfer(
    deps: DepsMut,
    from: String,
    to: String,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let coins = vec![Coin {
        denom,
        amount,
    }];
    let from_addr = deps.api.addr_validate(&from)?;
    let to_addr = deps.api.addr_validate(&to)?;
    let msgs = transfer(deps.storage, &from_addr, &to_addr, &coins)?;
    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "bank/force_transfer")
        .add_attribute("from", from)
        .add_attribute("to", to)
        .add_attribute("coin", stringify_coins(&coins)))
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
) -> Result<Vec<WasmMsg>, ContractError> {
    let mut msgs = vec![];

    for coin in coins {
        BALANCES.update(store, (from_addr, &coin.denom), |opt| {
            opt
                .unwrap_or_else(Uint128::zero)
                .checked_sub(coin.amount)
                .map_err(ContractError::from)
        })?;
        BALANCES.update(store, (to_addr, &coin.denom), |opt| {
            opt
                .unwrap_or_else(Uint128::zero)
                .checked_add(coin.amount)
                .map_err(ContractError::from)
        })?;

        let namespace = Namespace::extract_from_denom(&coin.denom)?;
        if let Some(namespace_cfg) = NAMESPACE_CONFIGS.may_load(store, &namespace)? {
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

    Ok(msgs)
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
    }
    Err(ContractError::not_namespace_admin(namespace))
}

fn stringify_coins(coins: &[Coin]) -> String {
    coins
        .iter()
        .map(|coin| coin.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn stringify_option(opt: Option<impl ToString>) -> String {
    opt.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn validate_optional_addr(api: &dyn Api, opt: Option<&String>) -> StdResult<Option<Addr>> {
    opt.map(|s| api.addr_validate(s)).transpose()
}
