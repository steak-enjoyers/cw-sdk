use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};

use cw_sdk::AddressLike;

use crate::denom::Namespace;

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

    /// Minter addresses and a namespaces they are allowed to mint.
    ///
    /// NOTE: There must be no duplication in namespaces.
    pub minters: Vec<Minter>,

    /// Initial balances of each account.
    ///
    /// NOTE:
    /// - There must be no duplication in addresses.
    /// - For each address, there must be no duplication of coin denoms.
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
    pub namespace: Namespace,
}

// TODO: this should be in `cw-sdk` crate
#[cw_serde]
pub enum SudoMsg {
    /// Forcibly transfer coins between designated accounts.
    /// Callable by the state machine when handling gas fee payments and funds attached to messages.
    Transfer {
        from: String,
        to: String,
        coins: Vec<Coin>,
    },
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Set the minting authorization for an account.
    /// Only callable by the contract owner or the namespace's current admin.
    UpdateNamespace {
        namespace: Namespace,
        admin: Option<String>,
        hookable: bool,
    },

    /// Send one or more coins to the given recipient.
    Send {
        to: String,
        coins: Vec<Coin>,
    },

    /// Mint a coin to the designated account's balance.
    /// Only callable by the namespace's admin.
    Mint {
        to: String,
        denom: String,
        amount: Uint128,
    },

    /// Burn a coin from the designated account's balance.
    /// Only callable by the namespace's admin.
    Burn {
        from: String,
        denom: String,
        amount: Uint128,
    },

    /// Forcibly transfer a coin between designated accounts.
    /// Only callable by the namespace's admin.
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
    /// Contract configurations
    #[returns(Config<String>)]
    Config {},

    /// Query the config of a single namespace
    #[returns(NamespaceResponse)]
    Namespace {
        namespace: Namespace,
    },

    /// Enumerate all namespaces
    #[returns(Vec<NamespaceResponse>)]
    Namespaces {
        start_after: Option<Namespace>,
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

#[cw_serde]
pub struct NamespaceResponse {
    pub namespace: Namespace,
    pub admin: Option<String>,
    pub hookable: bool,
}
