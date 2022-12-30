pub mod auth;
pub mod backend;
pub mod error;
pub mod execute;
pub mod query;
pub mod state;

use cosmwasm_std::{
    to_binary, Addr, Binary, BlockInfo, ContractInfo, Env, Event, MessageInfo, Storage, Timestamp,
    TransactionInfo,
};
use cw_sdk::{address, hash::HASH_LENGTH, GenesisState, SdkMsg, SdkQuery, Tx};
use cw_store::{Cached, Shared, Store};

use crate::{
    error::{Error, Result},
    state::{ACCOUNTS, BLOCK, CODE_COUNT},
};

pub struct StateMachine {
    /// The database backend, which stores blockchain state persistently.
    ///
    /// TODO: use a trait instead of a type, so that we can substitute this with
    /// a mock store for tests.
    store: Store,

    /// The block that is being processed and not yet commmitted to the store.
    ///
    /// The lifecycle of this variable is as follows:
    ///
    /// - Set to None when the application is booted;
    /// - During the BeginBlock ABCI method, it is set to Some using the
    ///   provided block height, time, and chain id;
    /// - During the Commit ABCI method, its value is committed to the chain's
    ///   state using the BLOCK storage constant.
    pending_block: Option<BlockInfo>,

    // TODO: load pinned contracts and codes
}

impl StateMachine {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            pending_block: None,
        }
    }

    /// Decode genesis bytes and run genesis messages. Return app hash.
    ///
    /// TODO: Once a staking contract is created, return the validator set as well
    pub fn init_chain(&self, chain_id: String, gen_state: GenesisState) -> Result<[u8; HASH_LENGTH]> {
        // make a cache of the store. only flush it if the entire init chain
        // flow is successful.
        // additionally, wrap the cached store in `Rc<RefCell<T>>` so that it
        // can be shared across the execution of multiple messages.
        let mut cache = Shared::new(Cached::new(self.store.pending_wrap()));

        let block = BlockInfo {
            height: 0,
            time: Timestamp::default(),
            chain_id,
        };

        BLOCK.save(&mut cache, &block)?;
        CODE_COUNT.save(&mut cache, &0)?;

        let deployer_addr = address::validate(&gen_state.deployer)?;

        // execute messages in order.
        // ResponseInitChain doesn't take events, so we discard the emitted events here.
        for msg in gen_state.msgs {
            self.handle_msg(
                cache.share(),
                block.clone(),
                None,
                &deployer_addr,
                msg,
            )?;
        }

        // init chain is successful; flush the state changes
        cache.borrow_mut().flush();

        Ok(self.store.root_hash())
    }

    pub fn begin_block(&mut self, block: BlockInfo) -> Result<Vec<Event>> {
        // current we just update pending block and do nothing else
        // TODO: read cosmos-sdk code and see what to do here
        self.pending_block = Some(block);

        Ok(vec![])
    }

    pub fn deliver_tx(&self, tx: Tx) -> Result<Vec<Event>> {
        // make a cache of the store. it will only be flushed if the entire tx
        // is successful
        let mut cache = Cached::new(self.store.pending_wrap());

        // authenticate signature, chain id, sequence, etc.
        let sender = auth::authenticate_tx(&cache, self.pending_block.as_ref().unwrap(), &tx)?;

        // update the sender's account in the store
        ACCOUNTS.save(&mut cache, &sender.address, &sender.account)?;

        // wrap the cached store in a `Rc<RefCell<T>>` so that it can be shared
        // as an owned value across the execution of multiple messages
        let mut cache = Shared::new(cache);

        let mut events = vec![];

        tx
            .body
            .msgs
            .into_iter()
            .map(|msg| {
                self.handle_msg(
                    cache.share(),
                    self.pending_block.clone().unwrap(),
                    None,
                    &sender.address,
                    msg,
                )
            })
            .try_for_each(|res| -> Result<_> {
                events.extend(res?);
                Ok(())
            })?;

        // tx is successful: flush the state changes
        cache.borrow_mut().flush();

        Ok(events)
    }

    fn handle_msg(
        &self,
        mut store: impl Storage + 'static,
        block: BlockInfo,
        transaction: Option<TransactionInfo>,
        sender_addr: &Addr,
        msg: SdkMsg,
    ) -> Result<Vec<Event>> {
        match msg {
            SdkMsg::StoreCode {
                wasm_byte_code,
            } => {
                let event = execute::store_code(&mut store, sender_addr, &wasm_byte_code)?;
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

                if !funds.is_empty() {
                    return Err(Error::FundsUnsupported);
                }
                let info = MessageInfo {
                    sender: sender_addr.clone(),
                    funds,
                };

                let result = execute::instantiate_contract(
                    store,
                    block,
                    transaction,
                    &info,
                    code_id,
                    &serde_json::to_vec(&msg)?,
                    label,
                    admin_addr,
                )?
                .into_result();

                if let Ok(res) = &result {
                    if !res.messages.is_empty() {
                        return Err(Error::SubmessagesUnsupported);
                    }
                }

                result.map(|res| res.events).map_err(Error::Contract)
            },
            SdkMsg::Execute {
                contract,
                msg,
                funds,
            } => {
                let env = Env {
                    block,
                    transaction,
                    contract: ContractInfo {
                        address: address::validate(&contract)?,
                    },
                };

                if !funds.is_empty() {
                    return Err(Error::FundsUnsupported);
                }
                let info = MessageInfo {
                    sender: sender_addr.clone(),
                    funds,
                };

                let result = execute::execute_contract(
                    store,
                    &env,
                    &info,
                    &serde_json::to_vec(&msg)?,
                )?
                .into_result();

                if let Ok(res) = &result {
                    if !res.messages.is_empty() {
                        return Err(Error::SubmessagesUnsupported);
                    }
                }

                result.map(|res| res.events).map_err(Error::Contract)
            },
            SdkMsg::Migrate {
                contract,
                code_id,
                msg,
            } => {
                let env = Env {
                    block,
                    transaction,
                    contract: ContractInfo {
                        address: address::validate(&contract)?,
                    },
                };

                let result = execute::migrate_contract(
                    store,
                    &env,
                    code_id,
                    &serde_json::to_vec(&msg)?,
                )?
                .into_result();

                if let Ok(res) = &result {
                    if !res.messages.is_empty() {
                        return Err(Error::SubmessagesUnsupported);
                    }
                }

                result.map(|res| res.events).map_err(Error::Contract)
            },
        }
    }

    pub fn info(&self) -> Result<(i64, [u8; HASH_LENGTH])> {
        let block = BLOCK.may_load(&self.store.wrap())?;
        let app_hash = self.store.root_hash();
        Ok((
            // when initializing a new chain scratch, Tendermint sends an Info
            // request prior to the InitChain request.
            // at this point the height hasn't been initialized yet. therefore
            // we do unwrap_or(0)
            block.map(|b| b.height as i64).unwrap_or(0),
            app_hash,
        ))
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
            SdkQuery::Contract {
                label
            } => to_binary(&query::contract(&store, label)?),
            SdkQuery::Contracts {
                start_after,
                limit,
            } => to_binary(&query::contracts(&store, start_after, limit)?),
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
            } => to_binary(&query::wasm_smart(store, &contract, &serde_json::to_vec(&msg)?)?),
        }
        .map_err(Error::from)
    }

    pub fn commit(&mut self) -> Result<(i64, [u8; HASH_LENGTH])> {
        // save the current pending block as the last committed block
        BLOCK.save(&mut self.store.pending_wrap(), self.pending_block.as_ref().unwrap())?;

        // clear the pending block
        self.pending_block = None;

        // commit pending ops to the underlying store
        self.store.commit()?;

        // return the block height and app hash that was just committed
        self.info()
    }
}
