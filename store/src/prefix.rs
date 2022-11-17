use cosmwasm_std::Storage;

#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, Record};

/// A helper function for creating a mutable prefixed store.
pub fn prefix(store: &mut dyn Storage, prefix: Vec<u8>) -> PrefixedStore {
    PrefixedStore {
        store,
        prefix,
    }
}

/// A helper function for creating a read-only prefixed store.
pub fn prefix_read(store: &dyn Storage, prefix: Vec<u8>) -> ReadonlyPrefixedStore {
    ReadonlyPrefixedStore {
        store,
        prefix,
    }
}

/// A storage object where all keys are prefixed by a certain byte array.
///
/// For example, if the prefix is `b"abc"`, and a user reads/writes from the key
/// `b"def"`, and actual key that will be used is `b"abcedf"`.
///
/// This is a _mutable_ prefixed store, meaning it needs a mutable reference of
/// the underlying store.
///
/// NOTE: Typically we need to also prefix the key `b"def"` by its length, e.g.
/// https://github.com/CosmWasm/cw-multi-test/blob/v0.16.0/src/prefixed_storage/length_prefixed.rs#L7-L14
/// However, inside cw-sdk, the only use case for a prefixed store is to get
/// contract substore, meaning the prefix is always `b"contract" + contract_addr`
/// which is of a known and fixed length, so we don't need the length prefix.
pub struct PrefixedStore<'a> {
    store: &'a mut dyn Storage,
    prefix: Vec<u8>,
}

impl<'a> Storage for PrefixedStore<'a> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.store.get(&concat(&self.prefix, key))
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.store.set(&concat(&self.prefix, key), value);
    }

    fn remove(&mut self, key: &[u8]) {
        self.store.remove(&concat(&self.prefix, key));
    }

    #[cfg(feature = "iterator")]
    fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'b> {
        range_with_prefix(self.store, &self.prefix, start, end, order)
    }
}

/// Similar to `Prefixed`, but only implements read methods, and takes only an
/// _immutable_ reference of the underlying store.
pub struct ReadonlyPrefixedStore<'a> {
    store: &'a dyn Storage,
    prefix: Vec<u8>,
}

impl<'a> Storage for ReadonlyPrefixedStore<'a> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.store.get(&concat(&self.prefix, key))
    }

    fn set(&mut self, _key: &[u8], _value: &[u8]) {
        unreachable!();
    }

    fn remove(&mut self, _key: &[u8]) {
        unreachable!();
    }

    #[cfg(feature = "iterator")]
    fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'b> {
        range_with_prefix(self.store, &self.prefix, start, end, order)
    }
}

/// Copied from cw-multi-test
/// https://github.com/CosmWasm/cw-multi-test/blob/v0.16.0/src/prefixed_storage/namespace_helpers.rs#L33-L59
#[cfg(feature = "iterator")]
fn range_with_prefix<'b>(
    store: &'b dyn Storage,
    prefix: &[u8],
    start: Option<&[u8]>,
    end: Option<&[u8]>,
    order: Order,
) -> Box<dyn Iterator<Item = Record> + 'b> {
    let start = match start {
        Some(bytes) => concat(prefix, bytes),
        None => prefix.to_vec(),
    };
    let end = match end {
        Some(bytes) => concat(prefix, bytes),
        None => prefix_upper_bound(prefix),
    };

    let prefix = prefix.to_vec();

    Box::new(store
        .range(Some(&start), Some(&end), order)
        .map(move |(k, v)| (trim(&prefix, &k), v)))
}

/// Copied from cw-multi-test:
/// https://github.com/CosmWasm/cw-multi-test/blob/v0.16.0/src/prefixed_storage/namespace_helpers.rs#L26-L31
#[inline]
fn concat(prefix: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = prefix.to_vec();
    k.extend_from_slice(key);
    k
}

/// Copied from cw-multi-test:
/// https://github.com/CosmWasm/cw-multi-test/blob/v0.16.0/src/prefixed_storage/namespace_helpers.rs#L61-L65
#[cfg(feature = "iterator")]
#[inline]
fn trim(prefix: &[u8], key: &[u8]) -> Vec<u8> {
    key[prefix.len()..].to_vec()
}

/// Returns a new vec of same length and last byte incremented by one
/// If last bytes are 255, we handle overflow up the chain.
/// If all bytes are 255, this returns wrong data - but that is never possible as a namespace
///
/// Copied from cw-multi-test:
/// https://github.com/CosmWasm/cw-multi-test/blob/v0.16.0/src/prefixed_storage/namespace_helpers.rs#L67-L83
#[cfg(feature = "iterator")]
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
        let mut store = MockStorage::new();

        let mut prefixed = prefix(&mut store, b"prefix".to_vec());
        prefixed.set(b"key1", b"value1");

        let prefixed = prefix_read(&store, b"prefix".to_vec());
        let value = prefixed.get(b"key1");
        assert_eq!(value, Some(b"value1".to_vec()));

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

        let prefixed = prefix_read(&store, b"prefix".to_vec());

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
