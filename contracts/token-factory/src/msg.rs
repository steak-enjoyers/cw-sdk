use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};

use crate::types::Config;

pub type InstantiateMsg = Config<String>;

#[cw_serde]
pub enum ExecuteMsg {
    /// Set the fee for creating new denoms.
    /// Only callable by the owner.
    SetFee {
        token_creation_fee: Option<Coin>,
    },

    /// Create a new token with the given subdenom.
    /// If there is a token creation fee, the message must include sufficient amount of coins.
    CreateToken {
        subdenom: String,
        /// Whereas admin can be removed later, it must be set during token creation
        admin: String,
        before_transfer_hook: Option<String>,
    },

    /// Update a token's admin account.
    /// Only callable by the current admin.
    SetAdmin {
        /// Set to `None` to remove the admin account, which permanently disables minting and
        /// burning of this token.
        admin: Option<String>,
    },

    /// Update a token's before transfer hook contract address.
    /// Only callable by the current admin.
    SetBeforeTransferHook {
        /// Set to `None` to remove the hook.
        before_transfer_hook: Option<String>,
    },

    /// Mint new tokens to the designated account
    Mint {
        to: String,
        subdenom: String,
        amount: Uint128,
    },

    /// Burn tokens from from designated account's balance
    Burn {
        from: String,
        subdenom: String,
        /// Set to `None` to burn the account's entire balance
        amount: Option<Uint128>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Query the contract's configurations
    #[returns(Config<String>)]
    Config {},

    /// Query a single token by denom
    #[returns(TokenResponse)]
    Token {
        denom: String,
    },

    /// Enumerate all tokens by denoms
    #[returns(Vec<TokenResponse>)]
    Tokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct TokenResponse {
    pub denom: String,
    pub admin: Option<String>,
    pub before_transfer_hook: Option<String>,
}
