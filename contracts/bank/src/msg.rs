use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};

use cw_sdk::AddressLike;

#[cw_serde]
pub struct Config<T: AddressLike> {
    pub owner: T,
}

#[cw_serde]
pub struct InstantiateMsg {
    /// The contract's owner.
    /// Typically this is set to a governance contract.
    pub owner: String,

    /// Initial balances of each account.
    ///
    /// NOTE:
    /// - There must be no duplication in addresses.
    /// - For each address, there must be no duplication of coin denoms.
    pub balances: Vec<Balance>,

    /// Configurations of namespaces.
    ///
    /// NOTE: There must be no duplication in namespaces.
    pub namespace_cfgs: Vec<UpdateNamespaceMsg>,
}

#[cw_serde]
pub struct Balance {
    pub address: String,
    pub coins: Vec<Coin>,
}

#[cw_serde]
pub struct UpdateNamespaceMsg {
    pub namespace: String,
    pub admin: Option<String>,
    pub after_transfer_hook: Option<String>,
}

// TODO: this should be in `cw-sdk` crate
#[cw_serde]
pub enum SudoMsg {
    /// Forcibly transfer coins between two accounts.
    /// Callable by the state machine when handling gas fee payments and funds attached to messages.
    Transfer {
        from: String,
        to: String,
        coins: Vec<Coin>,
    },
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Update the configuration of a namespace.
    /// Only callable by the contract owner or the namespace's current admin.
    UpdateNamespace(UpdateNamespaceMsg),

    /// Send one or more coins to the specified recipient.
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
pub enum HookMsg {
    /// After a coin transfer, if the namespace's `after_transfer_hook` is defined, the bank
    /// contract will send this message to that address.
    AfterTransfer {
        from: String,
        to: String,
        denom: String,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Contract configuration
    #[returns(Config<String>)]
    Config {},

    /// Query the config of a single namespace
    #[returns(NamespaceResponse)]
    Namespace {
        namespace: String,
    },

    /// Enumerate configs of all namespaces
    #[returns(Vec<NamespaceResponse>)]
    Namespaces {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// The total supply of a single coin
    #[returns(Coin)]
    Supply {
        denom: String,
    },

    /// Enumerate total supplies of all coins
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

pub type NamespaceResponse = UpdateNamespaceMsg;
