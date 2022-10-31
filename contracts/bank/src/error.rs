use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

use crate::denom::{DenomError, Namespace};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Overflow(#[from] OverflowError),

    #[error(transparent)]
    Denom(#[from] DenomError),

    #[error("sender is not the contract owner")]
    NotOwner,

    #[error("sender is not the admin for namespace {namespace}")]
    NotNamespaceAdmin {
        namespace: String,
    },

    #[error("duplicate {ty}: {value}")]
    Duplication {
        ty: String,
        value: String,
    },
}

impl ContractError {
    pub fn not_namespace_admin(namespace: &Namespace) -> Self {
        Self::NotNamespaceAdmin {
            namespace: namespace.to_string(),
        }
    }

    pub fn duplicate_denom(denom: impl Into<String>) -> Self {
        Self::Duplication {
            ty: "denom".into(),
            value: denom.into(),
        }
    }

    pub fn duplicate_namespace(namespace: impl Into<String>) -> Self {
        Self::Duplication {
            ty: "namespace".into(),
            value: namespace.into(),
        }
    }
}
