use std::collections::{HashMap, VecDeque};

use cosmwasm_std::{Addr, Binary, ContractResult, Order, Record, Storage, SystemResult};
use cosmwasm_vm::{BackendError, BackendResult, GasInfo, Querier};
use cw_store::{prefix, prefix_read, PrefixedStore, ReadonlyPrefixedStore};

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

pub struct BackendStore<T>
where
    T: Storage,
{
    store: T,
    iterators: HashMap<u32, Iter>,
}

impl<T> cosmwasm_vm::Storage for BackendStore<T>
where
    T: Storage,
{
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        (Ok(self.store.get(key)), GasInfo::free())
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        self.store.set(key, value);
        (Ok(()), GasInfo::free())
    }

    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        self.store.remove(key);
        (Ok(()), GasInfo::free())
    }

    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32> {
        let last_id: u32 = self
            .iterators
            .len()
            .try_into()
            .expect("[substore]: failed to cast iterator id into u32");
        let new_id = last_id + 1;

        let items = self
            .store
            .range(start, end, order)
            .collect();
        let iter = Iter::new(items);

        self.iterators.insert(new_id, iter);

        (Ok(new_id), GasInfo::free())
    }

    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>> {
        if let Some(iter) = self.iterators.get_mut(&iterator_id) {
            (Ok(iter.next()), GasInfo::free())
        } else {
            (Err(BackendError::iterator_does_not_exist(iterator_id)), GasInfo::free())
        }
    }
}

struct Iter {
    items: VecDeque<Record>,
}

impl Iter {
    pub fn new(items: VecDeque<Record>) -> Self {
        Self {
            items,
        }
    }
}

impl Iterator for Iter {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop_front()
    }
}


pub fn backend_store<'a>(
    store: &'a mut dyn Storage,
    contract_addr: &Addr,
) -> BackendStore<PrefixedStore<'a>> {
    BackendStore {
        store: prefix(store, contract_namespace(&contract_addr)),
        iterators: HashMap::new(),
    }
}

pub fn backend_store_read<'a>(
    store: &'a dyn Storage,
    contract_addr: &Addr,
) -> BackendStore<ReadonlyPrefixedStore<'a>> {
    BackendStore {
        store: prefix_read(store, contract_namespace(&contract_addr)),
        iterators: HashMap::new(),
    }
}

fn contract_namespace(contract_addr: &Addr) -> Vec<u8> {
    let mut namespace = b"contract".to_vec();
    namespace.extend(contract_addr.to_string().into_bytes());
    namespace
}
