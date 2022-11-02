use std::{fmt, str::FromStr};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Attribute, Coin, StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, PrimaryKey};

use super::{is_alphanumeric, starts_with_number, Denom, DenomError};

/// The maximum allowed length for namespaces.
///
/// This is 2 characters shorter than `denom::MAX_LEN`, because the full denom is at least two
/// characters longer than the namespace.
pub const MAX_NAMESPACE_LEN: usize = 126;

/// Namespace is wrapper of `String`, representing a validated namespace,
/// similar to how `cosmwasm_std::Addr` is a wrapper of `String` and represents a validated address.
#[cw_serde]
pub struct Namespace(String);

impl FromStr for Namespace {
    type Err = DenomError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > MAX_NAMESPACE_LEN {
            return Err(DenomError::illegal_length(s));
        }

        if starts_with_number(s) {
            return Err(DenomError::leading_number(s));
        }

        if !is_alphanumeric(s) {
            return Err(DenomError::not_alphanumeric(s));
        }

        Ok(Self(s.to_owned()))
    }
}

impl From<&Denom> for Namespace {
    // If the denom is validated, we can safely assume the namespace is also valid.
    // Therefore we don't need to validate it here.
    fn from(denom: &Denom) -> Self {
        let denom = denom.to_string();
        let parts: Vec<_> = denom.split('/').collect();
        match parts.len() {
            1 => Self("".to_owned()),
            _ => Self(parts[0].to_owned()),
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

impl From<Namespace> for String {
    fn from(ns: Namespace) -> Self {
        ns.0
    }
}

impl From<Namespace> for Attribute {
    fn from(ns: Namespace) -> Attribute {
        Attribute {
            key: "namespace".into(),
            value: ns.to_string(),
        }
    }
}

impl<'a> PrimaryKey<'a> for &Namespace {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = ();
    type SuperSuffix = ();

    fn key(&self) -> Vec<Key> {
        vec![Key::Ref(self.0.as_bytes())]
    }
}

impl KeyDeserialize for &Namespace {
    type Output = Namespace;

    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        String::from_utf8(value).map(Namespace).map_err(StdError::from)
    }
}

impl Namespace {
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into()
    }

    #[cfg(test)]
    pub fn unchecked(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Configuration of a namespace
#[cw_serde]
pub struct NamespaceConfig {
    /// The administrator of the namespace, who has the abililty to:
    ///
    /// - mint any amount of coins under the namespace to any account
    /// - burn any amount of coins under the namespace from any account
    /// - force transfer any amount of coins under the namespace between any two accounts
    ///
    /// Apparently, the admin is very powerful, and has the ability to rug.
    ///
    /// Typically, it is expected the admin be a contract that implements logics specifying under
    /// what circumstances the aforementioned privileged actions are to be executed. The contract is
    /// expected to be open source and audited by its community.
    ///
    /// See the `cw-token-factory` contract in this crate for an example of admin contract
    /// implementation.
    ///
    /// The admin can be set to `None`, in which case no one is able to mint/burn/force transfer.
    pub admin: Option<Addr>,

    /// If set to `true`, bank contract will invoke the admin contract with the
    /// `NamespaceAdminExecuteMsg::AfterTransfer` message (defined in this file below) following a
    /// coin transfer.
    pub after_send_hook: Option<Addr>,
}

/// This is the execute message that the admin contract is expected to implement if `hookable` is
/// set to `true`.
#[cw_serde]
pub enum NamespaceAdminExecuteMsg {
    AfterTransfer {
        from: String,
        to: String,
        coin: Coin,
    },
}

#[test]
fn from_str() {
    // invalid namespace: too long
    let s = "abcdef1234567890".repeat(9); // 16 * 9 = 144 characters
    assert_eq!(Namespace::from_str(&s), Err(DenomError::illegal_length(&s)));

    // invalid namespace: starts with a number
    let s = "123abc";
    assert_eq!(Namespace::from_str(s), Err(DenomError::leading_number(s)));

    // invalid namespace: contains non-alphanumeric characters
    let s = "a!@/bc";
    assert_eq!(Namespace::from_str(s), Err(DenomError::not_alphanumeric(s)));

    // valid namespaces
    assert!(Namespace::from_str("").is_ok());
    assert!(Namespace::from_str("a1B2c3D4").is_ok());
}

#[test]
fn from_denom() {
    assert_eq!(Namespace::from(&Denom::unchecked("abc")), Namespace::unchecked(""));
    assert_eq!(Namespace::from(&Denom::unchecked("abc/def/gh")), Namespace::unchecked("abc"));
}

#[test]
fn primary_key() {
    let namespace = Namespace::unchecked("abcd1234");
    let namespace_ref = &namespace;
    let path = namespace_ref.key();
    assert_eq!(path.len(), 1);
    assert_eq!(path[0].as_ref(), b"abcd1234");
}

#[test]
fn key_deserialize() {
    assert_eq!(
        <&Namespace>::from_vec(b"abcd1234".to_vec()).unwrap(),
        Namespace::unchecked("abcd1234"),
    );
}
