use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

use crate::helpers::{
    denom::{namespace_to_str, DenomError, Namespace},
    dup::DuplicateError,
};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Overflow(#[from] OverflowError),

    #[error(transparent)]
    Denom(#[from] DenomError),

    #[error(transparent)]
    Duplicate(#[from] DuplicateError),

    #[error("sender is not the contract owner")]
    NotOwner,

    #[error("sender does not have authorization to mint coins under namespace {namespace}")]
    NotMinter {
        namespace: String,
    },
}

impl ContractError {
    pub fn duplicate_denom(denom: impl Into<String>) -> Self {
        DuplicateError {
            ty: "denom".into(),
            value: denom.into(),
        }
        .into()
    }

    pub fn not_minter(namespace: &Namespace) -> Self {
        Self::NotMinter {
            namespace: namespace_to_str(namespace).into(),
        }
    }
}
