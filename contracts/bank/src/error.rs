use std::fmt::Display;

use cosmwasm_std::StdError;
use thiserror::Error;

use crate::denom::DenomError;

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Denom(#[from] DenomError),

    #[error("sender is not the contract owner")]
    NotOwner,

    #[error("sender is not the admin for namespace {namespace}")]
    NotNamespaceAdmin {
        namespace: String,
    },

    #[error("namespace `{namespace}` has not been configured")]
    NonExistNamespace {
        namespace: String,
    },

    #[error("duplicate {ty}: {value}")]
    Duplication {
        ty: String,
        value: String,
    },

    #[error("account {address} has zero initial balance for denom {denom}")]
    ZeroInitBalance {
        address: String,
        denom: String,
    },

    #[error("cannot mint/burn/transfer zero amount of denom {denom}")]
    ZeroAmount {
        denom: String,
    },
}

impl ContractError {
    pub fn not_namespace_admin(namespace: impl Display) -> Self {
        Self::NotNamespaceAdmin {
            namespace: namespace.to_string(),
        }
    }

    pub fn non_exist_namespace(namespace: impl Display) -> Self {
        Self::NonExistNamespace {
            namespace: namespace.to_string(),
        }
    }

    pub fn duplicate_balance(address: impl Display, denom: impl Display) -> Self {
        Self::Duplication {
            ty: "balance".into(),
            value: format!("account {address}, denom {denom}"),
        }
    }

    pub fn duplicate_namespace(namespace: impl Into<String>) -> Self {
        Self::Duplication {
            ty: "namespace".into(),
            value: namespace.into(),
        }
    }

    pub fn zero_init_balance(address: impl Into<String>, denom: impl Into<String>) -> Self {
        Self::ZeroInitBalance {
            address: address.into(),
            denom: denom.into(),
        }
    }

    pub fn zero_amount(denom: impl Into<String>) -> Self {
        Self::ZeroAmount {
            denom: denom.into(),
        }
    }
}
