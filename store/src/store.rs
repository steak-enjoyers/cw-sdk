use std::{collections::BTreeMap, iter, path::Path};

use cosmwasm_std::{Order, Record, Storage};
use merk::{Merk, Op};

use crate::{helpers::must_get, MerkError};
#[cfg(feature = "iterator")]
use crate::iterators::{range_bounds, Iter, MergedIter};

pub struct Store {
    /// The Merk tree which holds the key-value data.
    pub(crate) merk: Merk,

    /// Database operations from by BeginBlock, DeliverTx, and EndBlock
    /// executions, but not yet committed to the Merk store.
    ///
    /// Upon an ABCI "Commit" request, these ops will be committed to the Merk
    /// store, and this map cleared.
    pub(crate) pending_ops: BTreeMap<Vec<u8>, Op>,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MerkError> {
        Ok(Self {
            merk: Merk::open(path)?,
            pending_ops: BTreeMap::new(),
        })
    }

    /// Commit the pending changes to the underlying Merk store.
    /// This also writes the changes to disk, so should only be called during
    /// ABCI "Commit" requests.
    pub fn commit(&mut self) -> Result<(), MerkError> {
        // use `drain_filter` to clear the map and take ownership of all items.
        // this way we avoid having to clone the items
        let batch = self
            .pending_ops
            .drain_filter(|_, _| true)
            .collect::<Vec<_>>();

        unsafe {
            self.merk.apply_unchecked(&batch, &[])
        }
    }

    pub fn wrap(&self) -> StoreWrapper {
        StoreWrapper {
            store: self,
        }
    }

    pub fn wrap_mut(&mut self) -> PendingStoreWrapper {
        PendingStoreWrapper {
            store: self,
        }
    }
}

/// A read-only wrapper of the `Store` object, with the `cosmwasm_std::Storage`
/// trait implemented. When reading from this object, the underlying Merk store
/// is accessed, while the pending ops are ignored.
///
/// This struct is intended to be used in the ABCI "Query" request, so an
/// _immutable_ reference to the `Store` is used.
pub struct StoreWrapper<'a> {
    store: &'a Store,
}

impl<'a> Storage for StoreWrapper<'a> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        must_get(&self.store.merk, key)
    }

    fn set(&mut self, _key: &[u8], _value: &[u8]) {
        panic!("[cw-store]: `set` method invoked on read-only store wrapper");
    }

    fn remove(&mut self, _key: &[u8]) {
        panic!("[cw-store]: `remove` method invoked on read-only store wrapper");
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
        Box::new(Iter::new(&self.store.merk, start, end, order))
    }
}

/// A read-and-write wrapper of the `Store` object, with the `cosmwasm_std::Storage`
/// trait implemented. When reading or writing, the `pending_ops` map is accessed.
///
/// To be used in the following ABCI requests:
/// InitChain, BeginBlock, CheckTx, DeliverTx, EndBlock
pub struct PendingStoreWrapper<'a> {
    store: &'a mut Store,
}

impl<'a> Storage for PendingStoreWrapper<'a> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let Some(op) = self.store.pending_ops.get(key) else {
            return must_get(&self.store.merk, key);
        };
        match op {
            Op::Put(value) => Some(value.clone()),
            Op::Delete => None,
        }
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.store
            .pending_ops
            .insert(key.to_vec(), Op::Put(value.to_vec()));
    }

    fn remove(&mut self, key: &[u8]) {
        self.store
            .pending_ops
            .insert(key.to_vec(), Op::Delete);
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

        let base = Iter::new(&self.store.merk, start, end, order);

        let pending_raw = self.store.pending_ops.range(range_bounds(start, end));
        let pending: Box<dyn Iterator<Item = (&Vec<u8>, &Op)>> = match order {
            Order::Ascending => Box::new(pending_raw),
            Order::Descending => Box::new(pending_raw.rev()),
        };

        Box::new(MergedIter::new(base, pending, order))
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, time::SystemTime};

    use super::*;

    /// Open a `Store` at an autogenerated, temporary file path.
    /// Adapted from `merk::test_utils::TempMerk`:
    /// https://github.com/nomic-io/merk/blob/develop/src/test_utils/temp_merk.rs
    fn setup_test() -> Store {
        let mut path = temp_dir();
        let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        path.push(format!("merk-temp-{time}"));

        let mut store = Store::open(path).unwrap();

        // add some key-values for testing
        let batch = &[
            (b"key1".to_vec(), Op::Put(b"value1".to_vec())),
            (b"key2".to_vec(), Op::Put(b"value2".to_vec())),
            (b"key3".to_vec(), Op::Put(b"value3".to_vec())),
            (b"key4".to_vec(), Op::Put(b"value4".to_vec())),
        ];
        store.merk.apply(batch, &[]).unwrap();

        // add some pending ops as well
        let mut wrapper = store.wrap_mut();
        wrapper.set(b"key2", b"value23456");
        wrapper.set(b"key3333", b"value3333");
        wrapper.remove(b"key3");

        store
    }

    /// Delete a Merk database
    fn close_test(store: Store) {
        store.merk.destroy().expect("failed to delete db");
    }

    #[test]
    fn getting() {
        let mut store = setup_test();

        // read values from the read-only wrapper
        let wrapper = store.wrap();
        assert_eq!(wrapper.get(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(wrapper.get(b"key2"), Some(b"value2".to_vec()));
        assert_eq!(wrapper.get(b"key3"), Some(b"value3".to_vec()));
        assert_eq!(wrapper.get(b"key3333"), None);

        // read values from the pending wrapper
        let wrapper = store.wrap_mut();
        assert_eq!(wrapper.get(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(wrapper.get(b"key2"), Some(b"value23456".to_vec()));
        assert_eq!(wrapper.get(b"key3"), None);
        assert_eq!(wrapper.get(b"key3333"), Some(b"value3333".to_vec()));

        close_test(store);
    }

    #[test]
    fn committing() {
        let mut store = setup_test();

        store.commit().unwrap();

        let wrapper = store.wrap();
        assert_eq!(wrapper.get(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(wrapper.get(b"key2"), Some(b"value23456".to_vec()));
        assert_eq!(wrapper.get(b"key3"), None);
        assert_eq!(wrapper.get(b"key3333"), Some(b"value3333".to_vec()));

        // after committing, the pending ops should have been cleared
        assert!(store.pending_ops.is_empty());

        close_test(store);
    }

    #[test]
    #[should_panic = "[cw-store]: `set` method invoked on read-only store wrapper"]
    fn illegal_set() {
        let store = setup_test();

        let mut wrapper = store.wrap();
        wrapper.set(b"should", b"panic");
    }

    #[test]
    #[should_panic = "[cw-store]: `remove` method invoked on read-only store wrapper"]
    fn illegal_remove() {
        let store = setup_test();

        let mut wrapper = store.wrap();
        wrapper.remove(b"key2");
    }

    #[cfg(feature = "iterator")]
    #[test]
    fn iterating() {
        let store = setup_test();

        let mut kv = vec![
            (b"key1".to_vec(), b"value1".to_vec()),
            (b"key2".to_vec(), b"value2".to_vec()),
            (b"key3".to_vec(), b"value3".to_vec()),
            (b"key4".to_vec(), b"value4".to_vec()),
        ];

        // iterating with no bound and in ascending order
        let items = store.wrap().range(None, None, Order::Ascending).collect::<Vec<_>>();
        assert_eq!(items, kv);

        // iterating with bounds and in ascending order
        // NOTE: lower bound is inclusive, upper bound in exclusive
        let items = store
            .wrap()
            .range(Some(b"key1234"), Some(b"key4"), Order::Ascending)
            .collect::<Vec<_>>();
        assert_eq!(items, &kv[1..3]);

        kv.reverse();

        // iterating with no bound and in descending order
        let items = store.wrap().range(None, None, Order::Descending).collect::<Vec<_>>();
        assert_eq!(items, kv);

        // iterating with bounds and in descending order
        let items = store
            .wrap()
            .range(Some(b"key1234"), Some(b"key4"), Order::Descending)
            .collect::<Vec<_>>();
        assert_eq!(items, &kv[1..3]);

        close_test(store);
    }

    #[cfg(feature = "iterator")]
    #[test]
    fn iterating_pending() {
        let mut store = setup_test();

        let mut kv = vec![
            (b"key1".to_vec(), b"value1".to_vec()),
            (b"key2".to_vec(), b"value23456".to_vec()),
            (b"key3333".to_vec(), b"value3333".to_vec()),
            (b"key4".to_vec(), b"value4".to_vec()),
        ];

        // iterating with no bound and in ascending order
        let items = store.wrap_mut().range(None, None, Order::Ascending).collect::<Vec<_>>();
        assert_eq!(items, kv);

        // iterating with bounds and in ascending order
        // NOTE: lower bound is inclusive, upper bound in exclusive
        let items = store
            .wrap_mut()
            .range(Some(b"key1234"), Some(b"key4"), Order::Ascending)
            .collect::<Vec<_>>();
        assert_eq!(items, &kv[1..3]);

        kv.reverse();

        // iterating with no bound and in descending order
        let items = store.wrap_mut().range(None, None, Order::Descending).collect::<Vec<_>>();
        assert_eq!(items, kv);

        // iterating with bounds and in descending order
        let items = store
            .wrap_mut()
            .range(Some(b"key1234"), Some(b"key4"), Order::Descending)
            .collect::<Vec<_>>();
        assert_eq!(items, &kv[1..3]);

        close_test(store);
    }
}
