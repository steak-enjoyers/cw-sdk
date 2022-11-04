use cosmwasm_std::{Addr, Api};

use crate::{error::ContractError, msg::NAMESPACE};

pub(crate) fn parse_denom(api: &dyn Api, denom: &str) -> Result<(Addr, String), ContractError> {
    let parts: Vec<_> = denom.split('/').collect();

    if parts.len() != 3 {
        return Err(ContractError::incorrect_denom_format(denom));
    }

    if parts[0] != NAMESPACE {
        return Err(ContractError::incorrect_denom_namespace(denom));
    }

    let creator = parts[1];
    let nonce = parts[2];

    Ok((api.addr_validate(creator)?, nonce.to_owned()))
}
