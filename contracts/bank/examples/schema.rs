use cosmwasm_schema::write_api;

use cw_bank::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        sudo: SudoMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
