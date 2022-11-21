pub mod auth;
pub mod backend;
pub mod error;
pub mod execute;
pub mod query;
pub mod state;

use cosmwasm_std::{to_binary, Binary};
use cw_sdk::SdkQuery;
use cw_store::SharedStore;

use crate::error::{Error, Result};

pub struct StateMachine {
    store: SharedStore,
}

impl StateMachine {
    pub fn new(store: &SharedStore) -> Self {
        Self {
            store: store.share(),
        }
    }

    pub fn query(&self, query: SdkQuery) -> Result<Binary> {
        match query {
            SdkQuery::Info {} => to_binary(&query::info(&self.store)?),
            SdkQuery::Account {
                address,
            } => to_binary(&query::account(&self.store, address)?),
            SdkQuery::Accounts {
                start_after,
                limit,
            } => to_binary(&query::accounts(&self.store, start_after, limit)?),
            SdkQuery::Code {
                code_id,
            } => to_binary(&query::code(&self.store, code_id)?),
            SdkQuery::Codes {
                start_after,
                limit,
            } => to_binary(&query::codes(&self.store, start_after, limit)?),
            SdkQuery::WasmRaw {
                contract,
                key,
            } => to_binary(&query::wasm_raw(&self.store, &contract, &key)?),
            SdkQuery::WasmSmart {
                contract,
                msg,
            } => to_binary(&query::wasm_smart(&self.store, &contract, &msg)?),
        }
        .map_err(Error::from)
    }
}
