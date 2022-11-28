//! Each coin denom consists of two components: the "namespace" and the
//! "subdenom", separated by a forward slash (`/`).
//!
//! The namespace can be an empty string, in which case we say this is a
//! "top-level" denom, i.e. it is not namespaced.
//!
//! For example, consider the following denoms:
//!
//! | denom                         | namespace   | subdenom                |
//! | ----------------------------- | ----------- | ----------------------- |
//! | `uatom`                       | `""`        | `"uatom"`               |
//! | `ibc/1234ABCD`                | `"ibc"`     | `"1234ABCD"`            |
//! | `factory/osmo1234abcd/uastro` | `"factory"` | `"osmo1234abcd/uastro"` |
//!
//! This namespacing semantics allows us to efficiently manage coin minting
//! authorizations. Specifically, each minter account is granted minting power
//! under one or more namespaces.
//!
//! For example,
//! - the "ibc-transfer" contract may be granted minting power under the `"ibc"`
//!   namespace
//! - The "token-factory" contract may be granted minting power under the
//!   `"factory"` namespace

use std::{fmt, str::FromStr};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, PrimaryKey};

mod error;
mod namespace;

pub use error::*;
pub use namespace::*;

pub const MIN_LEN: usize = 3;
pub const MAX_LEN: usize = 128;

/// Denom is a wrapper of `String`, representing a validated coin denom,
/// similar to how `cosmwasm_std::Addr` is a wrapper of `String` and represents
/// a validated address.
#[cw_serde]
pub struct Denom(String);

impl Denom {
    pub fn validate(s: &str) -> Result<(), DenomError> {
        if starts_with_number(s) {
            return Err(DenomError::leading_number(s));
        }

        if !(MIN_LEN..=MAX_LEN).contains(&s.len()) {
            return Err(DenomError::illegal_length(s));
        }

        for part in s.split('/') {
            if part.is_empty() {
                return Err(DenomError::empty_parts(s));
            }

            if !is_alphanumeric(part) {
                return Err(DenomError::not_alphanumeric(s));
            }
        }

        Ok(())
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into()
    }

    #[cfg(test)]
    pub fn unchecked(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl FromStr for Denom {
    type Err = DenomError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s).map(|_| Self(s.to_owned()))
    }
}

impl fmt::Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Denom> for String {
    fn from(denom: Denom) -> Self {
        denom.0
    }
}

impl<'a> PrimaryKey<'a> for &Denom {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = ();
    type SuperSuffix = ();

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        vec![Key::Ref(self.0.as_bytes())]
    }
}

impl KeyDeserialize for &Denom {
    type Output = Denom;

    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        String::from_utf8(value).map(Denom).map_err(StdError::from)
    }
}

/// Return whether the string contains only alphanumeric characters.
/// Our definition of "alphanumeric" means within the following charset: 0-9|a-z|A-Z,
/// which is narrower than Unicode's definition, which Rust uses.
fn is_alphanumeric(s: &str) -> bool {
    s.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='z' | 'A'..='Z'))
}

/// Return whether the string starts with a number 0-9.
fn starts_with_number(s: &str) -> bool {
    s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}

#[test]
fn from_str() {
    // invalid denom: starts with a number
    let denom = "123abc/def";
    assert_eq!(Denom::from_str(denom), Err(DenomError::leading_number(denom)));

    // invalid denom: too short
    let denom = "ab";
    assert_eq!(Denom::from_str(denom), Err(DenomError::illegal_length(denom)));

    // invalid denom: too long
    let namespace = "abcedf1234567890".repeat(8); // 128 characters
    let subdenom = "abc";
    let denom = format!("{namespace}/{subdenom}");
    assert_eq!(Denom::from_str(&denom), Err(DenomError::illegal_length(denom)));

    // invalid denom: contains empty parts
    let denom = "/ccc";
    assert_eq!(Denom::from_str(denom), Err(DenomError::empty_parts(denom)));
    let denom = "ab/";
    assert_eq!(Denom::from_str(denom), Err(DenomError::empty_parts(denom)));
    let denom = "ab//c";
    assert_eq!(Denom::from_str(denom), Err(DenomError::empty_parts(denom)));

    // invalid denom: contains non-alphanumeric characters
    let denom = "ibc/1234@#$%abcd";
    assert_eq!(Denom::from_str(denom), Err(DenomError::not_alphanumeric(denom)));

    // valid denoms
    assert!(Denom::from_str("abc").is_ok());
    assert!(Denom::from_str("a/b").is_ok());
    assert!(Denom::from_str("az4Z5z/b1C2d/e2E3e4E5e6").is_ok());
}

#[test]
fn primary_key() {
    let denom = "a/b/c/d";
    let denom_ref = &denom;
    let path = denom_ref.key();
    assert_eq!(path.len(), 1);
    assert_eq!(path[0].as_ref(), b"a/b/c/d");
}

#[test]
fn key_deserialize() {
    assert_eq!(<&Denom>::from_vec(b"a/b/c/d".to_vec()).unwrap(), Denom::unchecked("a/b/c/d"));
}
