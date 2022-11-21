pub mod auth;
pub mod backend;
pub mod error;
pub mod execute;
pub mod query;
pub mod state;

use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, ContractResult, Env, Event, MessageInfo, Response,
    WasmMsg,
};
use cw_sdk::{address, SdkMsg, SdkQuery, Tx};
use cw_store::{Cached, Shared, Store};
use state::ACCOUNTS;

use crate::error::{Error, Result};

pub struct StateMachine {
    store: Store,
}

impl StateMachine {
    pub fn new(store: &Store) -> Self {
        Self {
            store: store.share(),
        }
    }

    pub fn deliver_tx(&self, tx: Tx) -> Result<Vec<Event>> {
        // make a cache of the store. it will only be flushed if the entire tx
        // is successful
        let cache = Cached::new(self.store.pending_wrap());

        // authenticate signature, chain id, sequence, etc.
        let sender = auth::authenticate_tx(&cache, &tx)?;

        // update the sender's account in the store
        ACCOUNTS.save(&mut cache, &sender.address, &sender.account)?;

        // wrap the cached store in a `Rc<RefCell<T>>` so that it can be shared
        // as an owned value across the execution of multiple messages
        let cache = Shared::new(cache);

        let mut events = vec![];

        tx.body
            .msgs
            .into_iter()
            .map(|msg| match msg {
                SdkMsg::StoreCode {
                    wasm_byte_code,
                } => {
                    let event = execute::store_code(&mut cache, &sender.address, &wasm_byte_code)?;
                    Ok(vec![event])
                },
                SdkMsg::Instantiate {
                    code_id,
                    msg,
                    funds,
                    label,
                    admin,
                } => {
                    let admin_addr = admin.map(|admin| address::validate(&admin)).transpose()?;
                    execute::instaniate_contract(
                        &sender.address,
                        code_id,
                        msg.into(),
                        funds,
                        label,
                        admin_addr,
                        AddressGenerator::ByIds,
                    )
                },
                SdkMsg::Execute {
                    contract,
                    msg,
                    funds,
                } => {
                    let contract_addr = address::validate(&contract)?;
                    execute::execute_contract(&sender.address, contract_addr, msg.into(), funds)
                },
                SdkMsg::Migrate {
                    contract,
                    code_id,
                    msg,
                } => {
                    let contract_addr = address::validate(&contract)?;
                    execute::migrate_contract(&sender.address, contract_addr, code_id, msg.into())
                },
            })
            .try_for_each(|res| {
                events.extend(res?);
                Ok(())
            })?;

        // tx is successful: flush the committed changes
        cache.borrow_mut().flush();

        Ok(events)
    }

    pub fn query(&self, query: SdkQuery) -> Result<Binary> {
        let store = self.store.wrap();
        match query {
            SdkQuery::Info {} => to_binary(&query::info(&store)?),
            SdkQuery::Account {
                address,
            } => to_binary(&query::account(&store, address)?),
            SdkQuery::Accounts {
                start_after,
                limit,
            } => to_binary(&query::accounts(&store, start_after, limit)?),
            SdkQuery::Code {
                code_id,
            } => to_binary(&query::code(&store, code_id)?),
            SdkQuery::Codes {
                start_after,
                limit,
            } => to_binary(&query::codes(&store, start_after, limit)?),
            SdkQuery::WasmRaw {
                contract,
                key,
            } => to_binary(&query::wasm_raw(store, &contract, &key)?),
            SdkQuery::WasmSmart {
                contract,
                msg,
            } => to_binary(&query::wasm_smart(store, &contract, &msg)?),
        }
        .map_err(Error::from)
    }
}

/// Represents which algorithm to use to derive contract addresses during instantiation.
enum AddressGenerator {
    /// Used during chain genesis
    ByLabel,
    /// Used post-genesis
    ByIds,
}
