use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

/// The instantiate message is inspired by the x/bank module's genesis state:
/// https://github.com/cosmos/cosmos-sdk/blob/v0.46.1/proto/cosmos/bank/v1beta1/genesis.proto
#[cw_serde]
pub struct InstantiateMsg {
    pub balances: Vec<Balance>,
}

#[cw_serde]
pub struct Balance {
    pub address: String,
    pub coins: Vec<Coin>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {
        to: String,
        amount: Coin,
    },
    Send {
        to: String,
        amount: Vec<Coin>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
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
