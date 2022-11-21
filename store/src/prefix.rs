use cosmwasm_std::{Order, Record, Storage};

/// A storage object where all keys are prefixed by a certain byte array.
///
/// For example, if the prefix is `b"abc"`, and a user reads/writes from the key
/// `b"def"`, and actual key that will be used is `b"abcedf"`.
///
/// Adapted from cw-storage-plus:
/// https://github.com/CosmWasm/cw-multi-test/blob/v0.16.0/src/prefixed_storage.rs
pub struct PrefixedStore<T: Storage> {
    store: T,
    prefix: Vec<u8>,
}

impl<T: Storage> PrefixedStore<T> {
    pub fn new(store: T, prefix: &[u8]) -> Self {
        Self {
            store,
            prefix: prefix.to_vec(),
        }
    }

    pub fn recycle(self) -> T {
        self.store
    }

    fn prefix(&self, key: &[u8]) -> Vec<u8> {
        let mut k = self.prefix.clone();
        k.extend_from_slice(key);
        k
    }

    fn trim(&self, prefixed_key: &[u8]) -> Vec<u8> {
        prefixed_key[self.prefix.len()..].to_vec()
    }
}

impl<T: Storage> Storage for PrefixedStore<T> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.store.get(&self.prefix(key))
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.store.set(&self.prefix(key), value);
    }

    fn remove(&mut self, key: &[u8]) {
        self.store.remove(&self.prefix(key));
    }

    fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'b> {
        let start = match start {
            Some(bytes) => self.prefix(bytes),
            None => self.prefix.clone(),
        };
        let end = match end {
            Some(bytes) => self.prefix(bytes),
            None => prefix_upper_bound(&self.prefix),
        };
        Box::new(self
            .store
            .range(Some(&start), Some(&end), order)
            .map(move |(k, v)| (self.trim(&k), v)))
    }
}

/// Returns a new vec of same length and last byte incremented by one
/// If last bytes are 255, we handle overflow up the chain.
/// If all bytes are 255, this returns wrong data - but that is never possible as a namespace
#[inline]
fn prefix_upper_bound(input: &[u8]) -> Vec<u8> {
    let mut copy = input.to_vec();
    // zero out all trailing 255, increment first that is not such
    for i in (0..input.len()).rev() {
        if copy[i] == 255 {
            copy[i] = 0;
        } else {
            copy[i] += 1;
            break;
        }
    }
    copy
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MockStorage;

    use super::*;

    #[test]
    fn prefixing() {
        let mut prefixed = PrefixedStore::new(MockStorage::new(), b"prefix");
        prefixed.set(b"key1", b"value1");

        let store = prefixed.recycle();
        let raw_value = store.get(b"prefixkey1");
        assert_eq!(raw_value, Some(b"value1".to_vec()));
    }

    #[test]
    fn iterating() {
        let mut store = MockStorage::new();
        store.set(b"key1", b"value1");
        store.set(b"key2", b"value2");
        store.set(b"prefixkey3", b"value3");
        store.set(b"prefixkey3333", b"value3333");
        store.set(b"prefixkey4", b"value4");
        store.set(b"prefixkey5", b"value5");
        store.set(b"prefixkey6", b"value6");
        store.set(b"prefiykey7", b"value7");
        store.set(b"zzz", b"abc");

        let prefixed = PrefixedStore::new(store, b"prefix");

        let mut kv = vec![
            (b"key3".to_vec(), b"value3".to_vec()),
            (b"key3333".to_vec(), b"value3333".to_vec()),
            (b"key4".to_vec(), b"value4".to_vec()),
            (b"key5".to_vec(), b"value5".to_vec()),
            (b"key6".to_vec(), b"value6".to_vec()),
        ];

        // iterating with no bound and in ascending order
        let items = prefixed
            .range(None, None, Order::Ascending)
            .collect::<Vec<_>>();
        assert_eq!(items, kv);

        // iterating with bounds and in ascending order
        // NOTE: lower bound is inclusive, upper bound in exclusive
        let items = prefixed
            .range(Some(b"key3333"), Some(b"key6"), Order::Ascending)
            .collect::<Vec<_>>();
        assert_eq!(items, &kv[1..4]);

        kv.reverse();

        // iterating with no bound and in descending order
        let items = prefixed
            .range(None, None, Order::Descending)
            .collect::<Vec<_>>();
        assert_eq!(items, kv);

        // iterating with bounds and in descending order
        let items = prefixed
            .range(Some(b"key3333"), Some(b"key6"), Order::Descending)
            .collect::<Vec<_>>();
        assert_eq!(items, &kv[1..4]);
    }
}
