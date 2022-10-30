use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

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
        /// Whereas admin can be removed later, it must be set during token creation.
        admin: String,
        /// See the comments on `crate::types::Token` on what this hook is.
        before_transfer_hook: Option<String>,
    },

    /// Update a token's admin account.
    /// Only callable by the token's current admin.
    SetAdmin {
        denom: String,
        /// Set to `None` to remove the admin account, which permanently disables minting and
        /// burning of this token.
        admin: Option<String>,
    },

    /// Update a token's before transfer hook contract address.
    /// Only callable by the token's admin.
    SetBeforeTransferHook {
        denom: String,
        /// Set to `None` to remove the hook.
        before_transfer_hook: Option<String>,
    },

    /// Mint new tokens to the designated account.
    /// Only callable by the token's admin.
    Mint {
        to: String,
        amount: Coin,
    },

    /// Burn tokens from from designated account's balance.
    /// Only callable by the token's admin.
    Burn {
        from: String,
        amount: Coin,
    },

    /// Forcibly transfer tokens between two accounts.
    /// Only callable by the token's admin.
    ForceTransfer {
        from: String,
        to: String,
        amount: Coin,
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
