use cosmwasm_std::{Addr, Deps, Order, StdResult};
use cw_storage_plus::Bound;

use crate::{
    error::ContractError,
    helpers::parse_denom,
    msg::{Config, TokenResponse, NAMESPACE},
    state::{CONFIG, TOKEN_CONFIGS},
};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub fn config(deps: Deps) -> StdResult<Config<String>> {
    CONFIG.load(deps.storage).map(|cfg| Config {
        owner: cfg.owner.into(),
        bank: cfg.bank.into(),
        token_creation_fee: cfg.token_creation_fee,
    })
}

pub fn token(deps: Deps, denom: String) -> Result<TokenResponse, ContractError> {
    let (creator, nonce) = parse_denom(deps.api, &denom)?;
    let cfg = TOKEN_CONFIGS.load(deps.storage, (&creator, &nonce))?;
    Ok(TokenResponse {
        denom,
        admin: cfg.admin.map(String::from),
        after_transfer_hook: cfg.after_transfer_hook.map(String::from),
    })
}

pub fn tokens(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<TokenResponse>, ContractError> {
    // a little hack to circumvent rust borrow check
    let (creator, nonce): (Addr, String);
    let start = match start_after {
        Some(s) => {
            (creator, nonce) = parse_denom(deps.api, &s)?;
            Some(Bound::exclusive((&creator, nonce.as_str())))
        },
        None => None,
    };

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    TOKEN_CONFIGS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let ((creator, nonce), cfg) = item?;
            Ok(TokenResponse {
                denom: format!("{NAMESPACE}/{creator}/{nonce}"),
                admin: cfg.admin.map(String::from),
                after_transfer_hook: cfg.after_transfer_hook.map(String::from),
            })
        })
        .collect()
}
