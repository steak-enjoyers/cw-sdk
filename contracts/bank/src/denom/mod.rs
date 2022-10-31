//! Each coin denom consists of two components: the "namespace" and the "subdenom", separated by a
//! forward slash (`/`).
//!
//! The namespace is optional. If the namespace is `None`, we say this is a "top-level" denom,
//! i.e. it is not namespaced.
//!
//! For example, consider the following denoms:
//!
//! | denom                         | namespace         | subdenom                |
//! | ----------------------------- | ----------------- | ----------------------- |
//! | `uatom`                       | `None`            | `"uatom"`               |
//! | `ibc/1234ABCD`                | `Some("ibc")`     | `"1234ABCD"`            |
//! | `factory/osmo1234abcd/uastro` | `Some("factory")` | `"osmo1234abcd/uastro"` |
//!
//! This namespacing semantics allows us to efficiently manage coin minting authorizations.
//! Specifically, each minter account is granted minting power under one or more namespaces.
//!
//! For example,
//! - the "ibc-transfer" contract may be granted minting power under the `"ibc"` namespace
//! - The "token-factory" contract may be granted minting power under the `"factory"` namespace

mod namespace;

pub use namespace::*;

pub const MIN_LEN: usize = 3;
pub const MAX_LEN: usize = 128;

/// Validate a denom. If valid, return the namespace and the subdenom.
/// This is typically called when handling the `mint` execute message.
pub fn validate_denom(denom: &str) -> Result<(), DenomError> {
    if starts_with_number(denom) {
        return Err(DenomError::leading_number(denom));
    }

    if !(MIN_LEN..=MAX_LEN).contains(&denom.len()) {
        return Err(DenomError::illegal_length(denom));
    }

    for part in denom.split('/') {
        if part.is_empty() {
            return Err(DenomError::empty_parts(denom));
        }

        if !is_alphanumeric(part) {
            return Err(DenomError::not_alphanumeric(denom));
        }
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
    s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}

#[derive(Debug, thiserror::Error)]
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
