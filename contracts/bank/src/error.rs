use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("unauthorized: {reason}")]
    Unauthorized {
        reason: String,
    },

    #[error("duplicate denom: {denom}")]
    DuplicateDenom {
        denom: String,
    },
}

impl ContractError {
    pub fn unauthorized(reason: impl ToString) -> Self {
        Self::Unauthorized {
            reason: reason.to_string(),
        }
    }

    pub fn duplicate_denom(denom: impl ToString) -> Self {
        Self::DuplicateDenom {
            denom: denom.to_string(),
        }
    }
}
