use cosmwasm_std::{
    to_binary, Addr, Coin, Deps, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};

use cw_bank::{denom::Denom, msg as bank};
use cw_sdk::helpers::{stringify_coins, stringify_option, validate_optional_addr};
use cw_utils::must_pay;

use crate::{
    error::ContractError,
    helpers::parse_denom,
    msg::{Config, TokenConfig, NAMESPACE},
    state::{CONFIG, TOKEN_CONFIGS},
};

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
    let cfg = CONFIG.update(deps.storage, |mut cfg| {
        if info.sender != cfg.owner {
            return Err(ContractError::NotOwner);
        }

        cfg.token_creation_fee = token_creation_fee;
        Ok(cfg)
    })?;

    Ok(Response::new()
        .add_attribute("action", "token-factory/update_fee")
        .add_attribute("new_fee", stringify_option(cfg.token_creation_fee)))
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

pub fn create_token(
    deps: DepsMut,
    info: MessageInfo,
    nonce: String,
    admin: String,
    after_transfer_hook: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    if let Some(expected) = cfg.token_creation_fee {
        let received_amount = must_pay(&info, &expected.denom)?;
        if received_amount != expected.amount {
            return Err(ContractError::incorrect_fee(expected, received_amount));
        }
    }

    let denom = format!("{NAMESPACE}/{}/{nonce}", &info.sender);
    Denom::validate(&denom)?;

    TOKEN_CONFIGS.update(deps.storage, (&info.sender, &nonce), |opt| {
        if opt.is_some() {
            return Err(ContractError::token_exists(&denom));
        }
        Ok(TokenConfig {
            admin: Some(deps.api.addr_validate(&denom)?),
            after_transfer_hook: validate_optional_addr(deps.api, after_transfer_hook.as_ref())?,
        })
    })?;

    Ok(Response::new()
        .add_attribute("action", "token-factory/create_token")
        .add_attribute("denom", denom)
        .add_attribute("admin", admin)
        .add_attribute("after_transfer_hook", stringify_option(after_transfer_hook)))
}

pub fn update_token(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
    admin: Option<String>,
    after_transfer_hook: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // Either the contract owner or the token's admin can update the config.
    // Here, if the sender is owner, we simply parse the denom into creator address + nonce,
    // and skip checking whether sender is the token admin.
    // If sender is not owner, we parse the denom AND check whether sender is the token admin.
    let (creator, nonce) = if info.sender == cfg.owner {
        parse_denom(deps.api, &denom)?
    } else {
        assert_denom_admin(deps.as_ref(), &denom, &info.sender)?
    };

    TOKEN_CONFIGS.update(deps.storage, (&creator, &nonce), |opt| -> Result<_, ContractError> {
        let mut token_cfg = opt.ok_or_else(|| ContractError::token_not_found(&denom))?;
        token_cfg.admin = validate_optional_addr(deps.api, admin.as_ref())?;
        token_cfg.after_transfer_hook = validate_optional_addr(deps.api,after_transfer_hook.as_ref())?;
        Ok(token_cfg)
    })?;

    Ok(Response::new()
        .add_attribute("action", "token-factory/update_token")
        .add_attribute("denom", denom)
        .add_attribute("admin", stringify_option(admin))
        .add_attribute("after_transfer_hook", stringify_option(after_transfer_hook)))
}

pub fn mint(
    deps: DepsMut,
    info: MessageInfo,
    to: String,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    assert_denom_admin(deps.as_ref(), &denom, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "token-factory/mint")
        .add_attribute("to", &to)
        .add_attribute("coin", format!("{amount}{denom}"))
        .add_message(WasmMsg::Execute {
            contract_addr: cfg.bank.into(),
            msg: to_binary(&bank::ExecuteMsg::Mint {
                to,
                denom,
                amount,
            })?,
            funds: vec![],
        }))
}

pub fn burn(
    deps: DepsMut,
    info: MessageInfo,
    from: String,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    assert_denom_admin(deps.as_ref(), &denom, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "token-factory/burn")
        .add_attribute("from", &from)
        .add_attribute("coin", format!("{amount}{denom}"))
        .add_message(WasmMsg::Execute {
            contract_addr: cfg.bank.into(),
            msg: to_binary(&bank::ExecuteMsg::Burn {
                from,
                denom,
                amount,
            })?,
            funds: vec![],
        }))
}

pub fn force_transfer(
    deps: DepsMut,
    info: MessageInfo,
    from: String,
    to: String,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    assert_denom_admin(deps.as_ref(), &denom, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "token-factory/burn")
        .add_attribute("from", &from)
        .add_attribute("coin", format!("{amount}{denom}"))
        .add_message(WasmMsg::Execute {
            contract_addr: cfg.bank.into(),
            msg: to_binary(&bank::ExecuteMsg::ForceTransfer {
                from,
                to,
                denom,
                amount,
            })?,
            funds: vec![],
        }))
}

pub fn after_transfer(
    deps: DepsMut,
    info: MessageInfo,
    from: String,
    to: String,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    if info.sender != cfg.bank {
        return Err(ContractError::NotBank);
    }

    let (creator_addr, nonce) = parse_denom(deps.api, &denom)?;
    let token_cfg = TOKEN_CONFIGS.load(deps.storage, (&creator_addr, &nonce))?;

    // do nothing if `after_transfer_hook` is not set for this denom
    let Some(after_transfer_hook) = token_cfg.after_transfer_hook else {
        return Ok(Response::default());
    };

    Ok(Response::new()
        .add_attribute("action", "token-factory/after_transfer")
        .add_attribute("from", &from)
        .add_attribute("to", &to)
        .add_attribute("coin", format!("{amount}{denom}"))
        .add_message(WasmMsg::Execute {
            contract_addr: after_transfer_hook.into(),
            msg: to_binary(&bank::HookMsg::AfterTransfer {
                from,
                to,
                denom,
                amount,
            })?,
            funds: vec![],
        }))
}

fn assert_denom_admin(
    deps: Deps,
    denom: &str,
    sender: &Addr,
) -> Result<(Addr, String), ContractError> {
    let (creator, nonce) = parse_denom(deps.api, denom)?;

    let Some(token_cfg) = TOKEN_CONFIGS.may_load(deps.storage, (&creator, &nonce))? else {
        return Err(ContractError::token_not_found(denom));
    };

    let Some(admin) = token_cfg.admin else {
        return Err(ContractError::not_token_admin(denom));
    };

    if *sender != admin {
        return Err(ContractError::not_token_admin(denom));
    }

    Ok((creator, nonce))
}
