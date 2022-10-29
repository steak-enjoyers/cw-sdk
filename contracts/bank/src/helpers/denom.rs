

use cosmwasm_std::{attr, Attribute};
use thiserror::Error;

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
/// For example, the "ibc-transfer" contract may be granted minting power under the `"ibc"`
/// namespace. The "token-factory" contract may be granted minting power under the `"factory"`
/// namespace.
pub struct Denom {
    pub namespace: Namespace,
    pub subdenom: String,
}

/// See the comment for `Denom` for an explainer of denom namespacing.
pub type Namespace = Option<String>;

/// Validate a denom. If valid, return the namespace and the subdenom.
/// This is typically called when handling the `mint` execute message.
pub fn validate_denom(denom: &str) -> Result<Denom, DenomError> {
    if starts_with_number(denom) {
        return Err(DenomError::leading_number(denom));
    }

    let len = denom.len();
    if !(MIN_LEN..=MAX_LEN).contains(&len) {
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

/// Validate a namespace.
/// This is typically called during instantiation and when handling the `set_minter` execute msg.
pub fn validate_namespace(namespace: &Namespace) -> Result<(), DenomError> {
    let namespace = match namespace {
        Some(ns) => ns,
        None => return Ok(()),
    };

    if namespace.is_empty() {
        return Err(DenomError::empty_parts(namespace));
    }

    if namespace.len() > MAX_LEN - 2 {
        return Err(DenomError::illegal_length(namespace));
    }

    if starts_with_number(namespace) {
        return Err(DenomError::leading_number(namespace));
    }

    if !is_alphanumeric(namespace) {
        return Err(DenomError::not_alphanumeric(namespace));
    }

    Ok(())
}

/// Return whether the string contains only alphanumeric characters.
/// Note that our definition of "alphanumeric" means within the following charset: 0-9|a-z|A-Z,
/// which is narrower than Unicode's definition, which Rust uses.
pub fn is_alphanumeric(s: &str) -> bool {
    s.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='z' | 'A'..='Z'))
}

/// Return whether the string starts with a number 0-9.
pub fn starts_with_number(s: &str) -> bool {
    s.chars().next().map(|ch| matches!(ch, '0'..='9')).unwrap_or(false)
}

/// Convert a `&Namespace` to a `&str` for use in error logging.
pub fn namespace_to_str(namespace: &Namespace) -> &str {
    match namespace {
        Some(ns) => ns,
        None => "null",
    }
}

/// Convert a `&Namespace` to `cosmwasm_std::Attribute` for use in event logging.
pub fn namespace_to_attr(namespace: &Namespace) -> Attribute {
    attr("namespace", namespace_to_str(namespace))
}

#[derive(Debug, Error)]
pub enum DenomError {
    #[error("invalid denom or namespace {denom}: contains empty parts")]
    EmptyParts {
        denom: String,
    },

    #[error("invalid denom or namespace {denom}: too long or too short")]
    IllegalLength {
        denom: String,
    },

    #[error("invalid denom or namespace {denom}: starts with a number")]
    LeadingNumber {
        denom: String,
    },

    #[error("invalid denom or namespace {denom}: contains non-alphanumeric characters")]
    NotAlphanumeric {
        denom: String,
    },
}

impl DenomError {
    pub fn empty_parts(denom: impl Into<String>) -> Self {
        Self::EmptyParts {
            denom: denom.into(),
        }
    }

    pub fn illegal_length(denom: impl Into<String>) -> Self {
        Self::IllegalLength {
            denom: denom.into(),
        }
    }

    pub fn leading_number(denom: impl Into<String>) -> Self {
        Self::LeadingNumber {
            denom: denom.into(),
        }
    }

    pub fn not_alphanumeric(denom: impl Into<String>) -> Self {
        Self::NotAlphanumeric {
            denom: denom.into(),
        }
    }
}
