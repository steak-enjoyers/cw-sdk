use cosmwasm_std::{attr, Attribute};
use thiserror::Error;

use super::Namespace;

pub const MIN_LEN: usize = 3;
pub const MAX_LEN: usize = 128;

/// Each coin denom consists of two components: the "namespace" and the "subdenom", separated by a
/// forward slash (`/`).
///
/// The namespace is optional. If the namespace is `None`, we say this is a "top-level" denom,
/// i.e. it is not namespaced.
///
/// For example, consider the following denoms:
///
/// | denom                         | namespace         | subdenom                |
/// | ----------------------------- | ----------------- | ----------------------- |
/// | `uatom`                       | `None`            | `"uatom"`               |
/// | `ibc/1234ABCD`                | `Some("ibc")`     | `"1234ABCD"`            |
/// | `factory/osmo1234abcd/uastro` | `Some("factory")` | `"osmo1234abcd/uastro"` |
///
/// This namespacing semantics allows us to efficiently manage coin minting authorizations.
/// Specifically, each minter account is granted minting power under one or more namespaces.
///
/// For example,
/// - the "ibc-transfer" contract may be granted minting power under the `"ibc"` namespace
/// - The "token-factory" contract may be granted minting power under the `"factory"` namespace
pub struct Denom {
    pub namespace: Namespace,
    pub subdenom: String,
}

/// Validate a denom. If valid, return the namespace and the subdenom.
/// This is typically called when handling the `mint` execute message.
pub fn validate_denom(denom: &str) -> Result<Denom, DenomError> {
    if starts_with_number(denom) {
        return Err(DenomError::leading_number(denom));
    }

    if !(MIN_LEN..=MAX_LEN).contains(&denom.len()) {
        return Err(DenomError::illegal_length(denom));
    }

    let parts: Vec<&str> = denom.split('/').collect();
    for part in &parts {
        if part.is_empty() {
            return Err(DenomError::empty_parts(denom));
        }

        if !is_alphanumeric(part) {
            return Err(DenomError::not_alphanumeric(denom));
        }
    }

    // no need to consider the case where parts.len() == 0
    // because we already asserted the denom's length > MIN_LEN
    match parts.len() {
        1 => Ok(Denom {
            namespace: None,
            subdenom: parts[0].into(),
        }),
        _ => Ok(Denom {
            namespace: Some(parts[0].into()),
            subdenom: parts[1..].join("/"),
        }),
    }
}
