//! This file is basically a copy of `cosmwasm_vm::testing::MockStorage`. The only difference is
//! that `MockStorage` doesn't implement the `Clone` trait but i kinda need it.
//!
//! In the future this will be replaced will be replaced with a real database backend.

use std::collections::{BTreeMap, HashMap};
use std::ops::{Bound, RangeBounds};

use cosmwasm_std::{Order, Record};
use cosmwasm_vm::{BackendError, BackendResult, GasInfo, Storage};

#[derive(Default, Debug, Clone)]
struct Iter {
    data: Vec<Record>,
    position: usize,
}

#[derive(Default, Debug, Clone)]
pub struct WasmStorage {
    pub data: BTreeMap<Vec<u8>, Vec<u8>>,
    iterators: HashMap<u32, Iter>,
}

impl WasmStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&mut self, iterator_id: u32) -> BackendResult<Vec<Record>> {
        let mut out: Vec<Record> = Vec::new();
        loop {
            let (result, _info) = self.next(iterator_id);
            match result {
                Err(err) => return (Err(err), GasInfo::free()),
                Ok(ok) => {
                    if let Some(v) = ok {
                        out.push(v);
                    } else {
                        break;
                    }
                },
            }
        }
        (Ok(out), GasInfo::free())
    }
}

impl Storage for WasmStorage {
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        (Ok(self.data.get(key).cloned()), GasInfo::free())
    }

    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32> {
        let bounds = range_bounds(start, end);

        let values: Vec<Record> = match (bounds.start_bound(), bounds.end_bound()) {
            // BTreeMap.range panics if range is start > end.
            // However, this cases represent just empty range and we treat it as such.
            (Bound::Included(start), Bound::Excluded(end)) if start > end => Vec::new(),
            _ => match order {
                Order::Ascending => self.data.range(bounds).map(clone_item).collect(),
                Order::Descending => self.data.range(bounds).rev().map(clone_item).collect(),
            },
        };

        let last_id: u32 =
            self.iterators.len().try_into().expect("Found more iterator IDs than supported");
        let new_id = last_id + 1;
        let iter = Iter {
            data: values,
            position: 0,
        };
        self.iterators.insert(new_id, iter);

        (Ok(new_id), GasInfo::free())
    }

    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>> {
        let iterator = match self.iterators.get_mut(&iterator_id) {
            Some(i) => i,
            None => {
                return (Err(BackendError::iterator_does_not_exist(iterator_id)), GasInfo::free())
            },
        };

        let value = if iterator.data.len() > iterator.position {
            let item = iterator.data[iterator.position].clone();
            iterator.position += 1;
            Some(item)
        } else {
            None
        };

        (Ok(value), GasInfo::free())
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        self.data.insert(key.to_vec(), value.to_vec());
        (Ok(()), GasInfo::free())
    }

    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        self.data.remove(key);
        (Ok(()), GasInfo::free())
    }
}

fn range_bounds(start: Option<&[u8]>, end: Option<&[u8]>) -> impl RangeBounds<Vec<u8>> {
    (
        start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
        end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
    )
}

/// The BTreeMap specific key-value pair reference type, as returned by BTreeMap<Vec<u8>, Vec<u8>>::range.
/// This is internal as it can change any time if the map implementation is swapped out.
type BTreeMapRecordRef<'a> = (&'a Vec<u8>, &'a Vec<u8>);

fn clone_item(item_ref: BTreeMapRecordRef) -> Record {
    let (key, value) = item_ref;
    (key.clone(), value.clone())
}
