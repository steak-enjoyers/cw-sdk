use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};

use cw_sdk::AddressLike;

#[cw_serde]
pub struct Config<T: AddressLike> {
    /// The contract's owner, who can call privileged methods
    pub owner: T,

    /// Address of the bank contract.
    /// NOTE: The bank contract must grant minting authorization of the "factory" namespace.
    pub bank: T,

    /// Address to which collected fees are to be sent to
    pub fee_collector: T,

    /// An optional fee for creating new denoms. Set to `None` to make it free.
    pub token_creation_fee: Option<Coin>,
}

#[cw_serde]
pub struct Token<T: AddressLike> {
    /// Admin is the account who can mint and burn tokens.
    /// Set this to `None` will permanently disable any burning or minting of this token.
    pub admin: Option<T>,

    /// A contract address that implements the following sudo message:
    ///
    /// ```rust
    /// enum SudoMsg {
    ///     BeforeTransfer {
    ///         from: Option<String>,
    ///         to: Option<String>,
    ///         amount: Uint128,
    ///     },
    /// }
    /// ```
    ///
    /// When the token is minted/burned/transferred, if set to `Some`, factory contract will
    /// invoke this method with the from/to addresses and amount.
    /// For minting, the `from` address is set to `None`.
    /// For burning, the `to` address is set to `None`.
    pub before_transfer_hook: Option<T>,
}

#[cw_serde]
pub enum HookSudoMsg {
    BeforeTransfer {
        from: Option<String>,
        to: Option<String>,
        amount: Uint128,
    },
}
