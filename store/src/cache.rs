use std::collections::BTreeMap;
#[cfg(feature = "iterator")]
use std::iter;

use cosmwasm_std::Storage;
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, Record};
use merk::Op;

#[cfg(feature = "iterator")]
use crate::iterators::{range_bounds, MergedIter};

/// A helper function for creating a cached store.
pub fn cache(store: &dyn Storage) -> CachedStore {
    CachedStore {
        store,
        pending_ops: BTreeMap::new(),
    }
}

/// Holds an immutable reference of any storage object that implements the
/// `Storage` trait, and a temporary, in-memory cache of uncommitted ops.
///
/// If the ops are to be committed to the underlying store, use the `prepare`
/// method to consume the cache, and then `flush`:
///
/// ```rust
/// use cosmwasm_std::{testing::MockStorage, Storage};
/// use cw_store::cache;
///
/// let mut store = MockStorage::new();
/// let mut cache = cache(&store);
///
/// cache.set(b"key1", b"value1");
/// cache.prepare().flush(&mut store);
/// ```
pub struct CachedStore<'a> {
    store: &'a dyn Storage,
    pending_ops: BTreeMap<Vec<u8>, Op>,
}

impl<'a> CachedStore<'a> {
    /// Consume self, return an array of ops ready to be flushed.
    pub fn prepare(self) -> Ops {
        Ops(self.pending_ops)
    }
}

// this block of code is basically duplicate from PendingStoreWrapper
// it'd be better if we can avoid duplication
impl<'a> Storage for CachedStore<'a> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let Some(op) = self.pending_ops.get(key) else {
            return self.store.get(key);
        };
        match op {
            Op::Put(value) => Some(value.clone()),
            Op::Delete => None,
        }
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.pending_ops.insert(key.to_vec(), Op::Put(value.to_vec()));
    }

    fn remove(&mut self, key: &[u8]) {
        self.pending_ops.insert(key.to_vec(), Op::Delete);
    }

    #[cfg(feature = "iterator")]
    fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'b> {
        if let (Some(start), Some(end)) = (start, end) {
            if start > end {
                return Box::new(iter::empty());
            }
        }

        let base = self.store.range(start, end, order);

        let pending_raw = self.pending_ops.range(range_bounds(start, end));
        let pending: Box<dyn Iterator<Item = (&Vec<u8>, &Op)>> = match order {
            Order::Ascending => Box::new(pending_raw),
            Order::Descending => Box::new(pending_raw.rev()),
        };

        Box::new(MergedIter::new(base, pending, order))
    }
}

/// Represents an array of ops ready to be flushed.
pub struct Ops(BTreeMap<Vec<u8>, Op>);

impl Ops {
    /// Commit the ops to the given store.
    ///
    /// We use the term "flush" instead of "commit" to differentiate with Store's
    /// commit method, which is only invoked during ABCI Commit requests.
    pub fn flush(self, base: &mut dyn Storage) {
        for (key, op) in self.0 {
            match op {
                Op::Put(value) => base.set(&key, &value),
                Op::Delete => base.remove(&key),
            }
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MockStorage;

    use super::*;

    fn setup_store(store: &mut MockStorage) {
        store.set(b"key1", b"value1");
        store.set(b"key2", b"value2");
        store.set(b"key3", b"value3");
        store.set(b"key4", b"value4");
    }

    fn setup_cache(cache: &mut CachedStore) {
        cache.set(b"key2", b"value23456");
        cache.set(b"key3333", b"value3333");
        cache.remove(b"key3");
    }

    fn kv() -> Vec<(Vec<u8>, Vec<u8>)> {
        vec![
            (b"key1".to_vec(), b"value1".to_vec()),
            (b"key2".to_vec(), b"value23456".to_vec()),
            (b"key3333".to_vec(), b"value3333".to_vec()),
            (b"key4".to_vec(), b"value4".to_vec()),
        ]
    }

    #[cfg(feature = "iterator")]
    #[test]
    fn flushing() {
        let mut store = MockStorage::default();
        setup_store(&mut store);

        let mut cache = cache(&store);
        setup_cache(&mut cache);

        cache.prepare().flush(&mut store);

        let items = store
            .range(None, None, Order::Ascending)
            .collect::<Vec<_>>();
        assert_eq!(items, kv());
    }

    #[cfg(feature = "iterator")]
    #[test]
    fn iterating() {
        let mut store = MockStorage::default();
        setup_store(&mut store);

        let mut cache = cache(&store);
        setup_cache(&mut cache);

        let mut kv = kv();

        // iterating with no bound and in ascending order
        let items = cache
            .range(None, None, Order::Ascending)
            .collect::<Vec<_>>();
        assert_eq!(items, kv);

        // iterating with bounds and in ascending order
        // NOTE: lower bound is inclusive, upper bound in exclusive
        let items = cache
            .range(Some(b"key1234"), Some(b"key4"), Order::Ascending)
            .collect::<Vec<_>>();
        assert_eq!(items, &kv[1..3]);

        kv.reverse();

        // iterating with no bound and in descending order
        let items = cache
            .range(None, None, Order::Descending)
            .collect::<Vec<_>>();
        assert_eq!(items, kv);

        // iterating with bounds and in descending order
        let items = cache
            .range(Some(b"key1234"), Some(b"key4"), Order::Descending)
            .collect::<Vec<_>>();
        assert_eq!(items, &kv[1..3]);
    }
}
