use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("sender is not the contract owner")]
    NotOwner,

    #[error("the contract has no coins to transfer")]
    NoBalance,
}
