use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Uint128};

use cw_sdk::AddressLike;

/// The namespace that the token factory contract must be assigned as admin at the bank contract.
pub const NAMESPACE: &str = "factory";

#[cw_serde]
pub struct Config<T: AddressLike> {
    /// The contract's owner
    pub owner: T,

    /// Address of the bank contract.
    pub bank: T,

    /// An optional fee for creating new denoms. Set to `None` to make it free.
    pub token_creation_fee: Option<Coin>,
}

#[cw_serde]
pub struct TokenConfig {
    /// Admin is the account who can mint and burn tokens.
    /// Set this to `None` will permanently disable any burning or minting of this token.
    pub admin: Option<Addr>,

    /// Any AfterSend hook message sent by the bank contract will be forwarded to this address.
    pub after_send_hook: Option<Addr>,
}

#[cw_serde]
pub struct UpdateTokenMsg {
    denom: String,
    admin: Option<String>,
    after_send_hook: Option<String>,
}

pub type InstantiateMsg = Config<String>;

#[cw_serde]
pub enum ExecuteMsg {
    /// Update the fee for creating new denoms.
    /// Only callable by the owner.
    UpdateFee {
        token_creation_fee: Option<Coin>,
    },

    /// Withdraw fees collected in the contract.
    /// Only callable by the owner.
    WithdrawFee {
        /// Address to which the fees are to be sent.
        /// Default to the owner if not provided.
        to: Option<String>,
    },

    /// Create a new token with the given nonce.
    /// If there is a token creation fee, the message must include sufficient amount of coins.
    CreateToken {
        nonce: String,

        /// We require that the admin must be specified during token creation.
        /// It doesn't make sense to create a token with no admin, because then no one would be able
        /// to ever mint it.
        /// However, the admin can be set to `None` later.
        admin: String,

        /// See the comments on `crate::types::Token` on what this hook is.
        after_send_hook: Option<String>,
    },

    /// Update a token's configuration.
    /// Only callable by the token's current admin.
    UpdateToken(UpdateTokenMsg),

    /// Mint new tokens to the designated account.
    /// Only callable by the token's admin.
    Mint {
        to: String,
        denom: String,
        amount: Uint128,
    },

    /// Burn tokens from from designated account's balance.
    /// Only callable by the token's admin.
    Burn {
        from: String,
        denom: String,
        amount: Uint128,
    },

    /// Forcibly transfer tokens between two accounts.
    /// Only callable by the token's admin.
    ForceTransfer {
        from: String,
        to: String,
        denom: String,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Query the contract's configurations
    #[returns(Config<String>)]
    Config {},

    /// Query the configuration of a single token by denom
    #[returns(TokenResponse)]
    Token {
        denom: String,
    },

    /// Enumerate the config of all tokens
    #[returns(Vec<TokenResponse>)]
    Tokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

pub type TokenResponse = UpdateTokenMsg;
