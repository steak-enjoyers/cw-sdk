use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Attribute, Coin, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, PrimaryKey};

use super::{is_alphanumeric, starts_with_number, DenomError};

/// The maximum allowed length for namespaces.
///
/// This is 2 characters shorter than `denom::MAX_LEN`, because the full denom is at least two
/// characters longer than the namespace.
pub const MAX_NAMESPACE_LEN: usize = 126;

/// Namespace is a prefix to denoms.
/// If it is `None`, we say the denom is "unnamespaced", or is a "top-level denom".
/// See the comments on `super::Denom` for a better explanation.
#[cw_serde]
pub struct Namespace(Option<String>);

impl Namespace {
    /// Extract the namespace from a denom.
    pub fn extract_from_denom(denom: &str) -> Result<Self, DenomError> {
        let parts: Vec<&str> = denom.split('/').collect();
        match parts.len() {
            1 => Ok(Self(None)),
            _ => {
                let namespace = Self(Some(parts[0].to_owned()));
                namespace.validate().map(|_| namespace)
            },
        }
    }

    /// Validate a namespace.
    /// Typically called during instantiation and when handling the `set_namespace` execute msg.
    pub fn validate(&self) -> Result<(), DenomError> {
        let ns = match &self.0 {
            Some(ns) => ns,
            None => return Ok(()),
        };

        if ns.is_empty() {
            return Err(DenomError::empty_parts(ns));
        }

        if ns.len() > MAX_NAMESPACE_LEN {
            return Err(DenomError::illegal_length(ns));
        }

        if starts_with_number(ns) {
            return Err(DenomError::leading_number(ns));
        }

        if !is_alphanumeric(ns) {
            return Err(DenomError::not_alphanumeric(ns));
        }

        Ok(())
    }
}

impl From<&Namespace> for String {
    fn from(namespace: &Namespace) -> Self {
        namespace.to_string()
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ns) => write!(f, "\"{ns}\""),
            None => write!(f, "null"),
        }
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
        let bytes = match &self.0 {
            Some(ns) => ns.as_bytes(),
            None => &[],
        };
        vec![Key::Ref(bytes)]
    }
}

impl KeyDeserialize for &Namespace {
    type Output = Namespace;

    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        if value.is_empty() {
            Ok(Namespace(None))
        } else {
            let ns = String::from_utf8(value)?;
            Ok(Namespace(Some(ns)))
        }
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
