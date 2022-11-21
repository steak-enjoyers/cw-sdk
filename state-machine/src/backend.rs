use std::collections::HashMap;

use cosmwasm_std::{Addr, Binary, ContractResult, Order, Record, Storage, SystemResult};
use cosmwasm_vm::{BackendError, BackendResult, GasInfo, Querier};

use cw_store::{
    iterators::MemIter,
    prefix::{concat, namespace_upper_bound, trim},
};

//--------------------------------------------------------------------------------------------------
// API
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct BackendApi;

impl cosmwasm_vm::BackendApi for BackendApi {
    // TODO: currently we just return the utf8 bytes of the string. in the future we should
    // implement proper bech32 decoding.
    fn canonical_address(&self, human: &str) -> BackendResult<Vec<u8>> {
        let bytes = human.as_bytes().to_owned();
        (Ok(bytes), GasInfo::free())
    }

    // TODO: currently we just return the utf8 bytes of the string. in the future we should
    // implement proper bech32 decoding.
    // a question here is, if this function is supposed to be stateless, how do we know which bech32
    // prefix to use? for Go SDK the prefix is hardcoded in the daemon, but for cw-sdk we don't want
    // to hardcode any chain-specific params.
    fn human_address(&self, canonical: &[u8]) -> BackendResult<String> {
        let human = String::from_utf8(canonical.to_owned())
            .map_err(|_| BackendError::user_err("invalid utf8"));
        (human, GasInfo::free())
    }
}

//--------------------------------------------------------------------------------------------------
// Querier
//--------------------------------------------------------------------------------------------------

pub struct BackendQuerier;

impl Querier for BackendQuerier {
    fn query_raw(
        &self,
        _request: &[u8],
        _gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        (Err(BackendError::user_err("`querier.query_raw` is not yet implemented")), GasInfo::free())
    }
}

//--------------------------------------------------------------------------------------------------
// Storage
//--------------------------------------------------------------------------------------------------

/// NOTE: cosmwasm-vm requires the backend store to be of 'static lifetime.
/// This requirement comes from wasmer so not something we can change.
///
/// We obviously can't borrow a reference of the store with static lifetime,
/// So it has to be an owned type.
///
/// Here we need both the `store` and `iterators` map be owned.
pub struct ContractSubstore<T: Storage> {
    store: T,
    namespace: Vec<u8>,
    iterators: HashMap<u32, MemIter>,
}

impl<T: Storage> ContractSubstore<T> {
    pub fn new(store: T, contract_addr: &Addr) -> Self {
        Self {
            store,
            namespace: contract_addr.to_string().into_bytes(),
            iterators: HashMap::new(),
        }
    }

    pub fn recycle(self) -> T {
        self.store
    }

    fn key(&self, k: &[u8]) -> Vec<u8> {
        concat(&self.namespace, k)
    }
}

impl<T: Storage> cosmwasm_vm::Storage for ContractSubstore<T> {
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        let value = self.store.get(&self.key(key));
        (Ok(value), GasInfo::free())
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        self.store.set(&self.key(key), value);
        (Ok(()), GasInfo::free())
    }

    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        self.store.remove(&self.key(key));
        (Ok(()), GasInfo::free())
    }

    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32> {
        let start = match start {
            Some(s) => concat(&self.namespace, s),
            None => self.namespace.to_vec(),
        };
        let end = match end {
            Some(e) => concat(&self.namespace, e),
            // end is updating last byte by one
            None => namespace_upper_bound(&self.namespace),
        };

        let iter = MemIter::new(self
            .store
            .range(Some(&start), Some(&end), order)
            .map(move |(k, v)| (trim(&self.namespace, &k), v)));
        let iter_count: u32 = self
            .iterators
            .len()
            .try_into()
            .expect("[substore]: failed to cast iterator id into u32");
        let iterator_id = iter_count + 1;

        self.iterators.insert(iterator_id, iter);

        (Ok(iterator_id), GasInfo::free())
    }

    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>> {
        if let Some(iter) = self.iterators.get_mut(&iterator_id) {
            (Ok(iter.next()), GasInfo::free())
        } else {
            (Err(BackendError::iterator_does_not_exist(iterator_id)), GasInfo::free())
        }
    }
}
