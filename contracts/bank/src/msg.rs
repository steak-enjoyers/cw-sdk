use std::collections::BTreeSet;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

use cw_sdk::AddressLike;

use crate::helpers::denom::Namespace;

#[cw_serde]
pub struct Config<T: AddressLike> {
    pub owner: T,
}

/// The instantiate message is inspired by the x/bank module's genesis state:
/// https://github.com/cosmos/cosmos-sdk/blob/v0.46.1/proto/cosmos/bank/v1beta1/genesis.proto
#[cw_serde]
pub struct InstantiateMsg {
    /// The account who can call privileged methods of the contract.
    /// Typically this is set to a governance contract.
    pub owner: String,

    /// Minter addresses and the namespaces they are allowed to mint.
    /// Check the comments of `Minter` struct for an explanation on denom namespacing.
    ///
    /// NOTE: There must be no duplication in addresses, and for each address,
    /// there must be no duplication in namespaces.
    pub minters: Vec<Minter>,

    /// Initial balances of each account.
    ///
    /// NOTE: There must be no duplication in addresses, and for each address,
    /// there must be no duplication of coin denoms.
    pub balances: Vec<Balance>,
}

#[cw_serde]
pub struct Balance {
    pub address: String,
    pub coins: Vec<Coin>,
}

#[cw_serde]
pub struct Minter {
    pub address: String,
    /// Denom namespaces under which this account is authorized to mint coins.
    pub namespaces: BTreeSet<Namespace>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Set the minting authorization for an account.
    /// Only callable by the owner.
    SetMinter {
        address: String,
        /// Namespaces under which that this minter is allowed to mint coins.
        /// There must be no duplications.
        namespaces: BTreeSet<Namespace>,
    },

    /// Mint a coin to the designated recipient.
    /// The minter must have been authorized to mint coins under the denom namespace.
    Mint {
        to: String,
        amount: Coin,
    },

    /// Send one or more coins to the given recipient.
    Send {
        to: String,
        amount: Vec<Coin>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Contract configurations
    #[returns(Config<String>)]
    Config {},

    /// Query a single minter by address
    #[returns(Minter)]
    Minter {
        address: String,
    },

    /// Enumerate all minters
    #[returns(Vec<Minter>)]
    Minters {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// The total supply of a single coin
    #[returns(Coin)]
    Supply {
        denom: String,
    },

    /// The total supply of all coins
    #[returns(Vec<Coin>)]
    Supplies {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// The balance of a single coin for a single account
    #[returns(Coin)]
    Balance {
        address: String,
        denom: String,
    },

    /// The balances of all coins for a single account
    #[returns(Vec<Coin>)]
    Balances {
        address: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
