use cosmwasm_std::{Addr, Api};

use crate::{error::ContractError, msg::NAMESPACE};

pub(crate) fn parse_denom(api: &dyn Api, denom: &str) -> Result<(Addr, String), ContractError> {
    let Some((namespace, subdenom)) = denom.split_once('/') else {
        return Err(ContractError::incorrect_denom_format(denom));
    };

    if namespace != NAMESPACE {
        return Err(ContractError::incorrect_denom_namespace(denom));
    }

    let Some((creator, nonce)) = subdenom.split_once('/') else {
        return Err(ContractError::incorrect_denom_format(denom));
    };

    Ok((api.addr_validate(creator)?, nonce.to_owned()))
}

#[cfg(test)]
use cosmwasm_std::testing::MockApi;

#[test]
fn parsing_denom() {
    let api = MockApi::default();

    let denom = "uastro";
    assert_eq!(parse_denom(&api, denom), Err(ContractError::incorrect_denom_format(denom)));

    let denom = "factory/uastro";
    assert_eq!(parse_denom(&api, denom), Err(ContractError::incorrect_denom_format(denom)));

    let denom = "astro/osmo1234abcd/uastro";
    assert_eq!(parse_denom(&api, denom), Err(ContractError::incorrect_denom_namespace(denom)));

    let denom = "factory/osmo1234abcd/NSFdn7sgfs97m0dNFU";
    assert_eq!(
        parse_denom(&api, denom),
        Ok((Addr::unchecked("osmo1234abcd"), "NSFdn7sgfs97m0dNFU".into())),
    );
}
