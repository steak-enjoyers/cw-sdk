use std::{
    cmp::Ordering,
    collections::VecDeque,
    iter::Peekable,
    ops::{Bound, RangeBounds},
};

use cosmwasm_std::{Order, Record};
use merk::{Merk, Op};

use crate::helpers::must_get;

/// Iterator over a Merk store.
///
/// Different from IAVL tree, the Merk tree stores raw keys as database keys.
/// To iterate keys in the tree, we simply iterate keys in the underlying RocksDB.
pub(crate) struct MerkIter<'a> {
    merk: &'a Merk,
    iter: rocksdb::DBRawIterator<'a>,
    start: Option<Vec<u8>>,
    end: Option<Vec<u8>>,
    order: Order,
    started: bool,
}

impl<'a> MerkIter<'a> {
    pub fn new(merk: &'a Merk, start: Option<&[u8]>, end: Option<&[u8]>, order: Order) -> Self {
        Self {
            merk,
            iter: merk.raw_iter(),
            start: start.map(|bytes| bytes.to_vec()),
            end: end.map(|bytes| bytes.to_vec()),
            order,
            started: false,
        }
    }
}

impl<'a> Iterator for MerkIter<'a> {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        if self.started {
            match self.order {
                Order::Ascending => self.iter.next(),
                Order::Descending => self.iter.prev(),
            }
        } else {
            match self.order {
                Order::Ascending => match &self.start {
                    Some(start) => self.iter.seek(start),
                    None => self.iter.seek_to_first(),
                },
                Order::Descending => match &self.end {
                    Some(end) => {
                        self.iter.seek_for_prev(end);
                        // end is exclusive, so if the current key matches end,
                        // we need to move back one
                        if let Some(key) = self.iter.key() {
                            if key == end {
                                self.iter.prev();
                            }
                        }
                    },
                    None => self.iter.seek_to_last(),
                },
            }
            self.started = true;
        }

        let Some(key) = self.iter.key() else {
            return None;
        };

        // determine whether the key is out of bound
        match self.order {
            Order::Ascending => {
                if let Some(end) = &self.end {
                    // NOTE: end is exclusive
                    if key >= end {
                        return None;
                    }
                }
            },
            Order::Descending => {
                if let Some(start) = &self.start {
                    // NOTE: start is inclusive
                    if key < start {
                        return None;
                    }
                }
            },
        }

        // two invariants:
        // - the read from Merk store must be successful
        // - if the key exists, the value must also exist (must not be a `None`)
        //
        // if either is violated, we consider it a fatal error, and panic.
        let value = must_get(self.merk, key).unwrap_or_else(|| {
            panic!(
                "[cw-store]: key {} exists but a corresponding value doesn't exist",
                hex::encode(key),
            );
        });

        Some((key.to_vec(), value))
    }
}

/// A "merged" iterator over both a base KV store and a cache of pending ops.
///
/// Adapted from `cw_multi_test::transactions::MergeOverlay`:
/// https://github.com/CosmWasm/cw-multi-test/blob/v0.16.0/src/transactions.rs#L179-L188
pub(crate) struct MergedIter<'a, B, P>
where
    B: Iterator<Item = Record>,
    P: Iterator<Item = (&'a Vec<u8>, &'a Op)>,
{
    base: Peekable<B>,
    pending: Peekable<P>,
    order: Order,
}

impl<'a, B, P> MergedIter<'a, B, P>
where
    B: Iterator<Item = Record>,
    P: Iterator<Item = (&'a Vec<u8>, &'a Op)>,
{
    pub fn new(base: B, pending: P, order: Order) -> Self {
        Self {
            base: base.peekable(),
            pending: pending.peekable(),
            order,
        }
    }

    /// If the pending op is to add a new KV, then return this KV.
    /// Otherwise, this key does not exist, advance to the next iteration step.
    fn take_pending(&mut self) -> Option<Record> {
        // `take_pending` is only called if we have peeked that `self.pending.next()`
        // will return `Some`, so we can safely unwrap here
        let (key, op) = self.pending.next().unwrap();
        match op {
            Op::Put(value) => Some((key.clone(), value.clone())),
            Op::Delete => self.next(),
        }
    }
}

impl<'a, B, P> Iterator for MergedIter<'a, B, P>
where
    B: Iterator<Item = Record>,
    P: Iterator<Item = (&'a Vec<u8>, &'a Op)>,
{
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.base.peek(), self.pending.peek()) {
            (Some(base_item), Some(pending_item)) => {
                let (base_key, _) = base_item;
                let (pending_key, _) = pending_item;

                // compare the keys of the base item and the pending op item
                // Ordering::Less means base precedes pending op,
                // Ordering::Greater means pending op precedes base
                let order_raw = base_key.cmp(pending_key);
                let order = match self.order {
                    Order::Ascending => order_raw,
                    Order::Descending => order_raw.reverse(),
                };

                match order {
                    Ordering::Less => self.base.next(),
                    Ordering::Equal => {
                        self.base.next();
                        self.take_pending()
                    },
                    Ordering::Greater => self.take_pending(),
                }
            },

            // base has reached end, pending has not
            (None, Some(_)) => self.take_pending(),

            // pending has reached end, base has not => simply return next base
            (Some(_), None) => self.base.next(),

            // both base and pending have reached end => simply return None
            (None, None) => None,
        }
    }
}

/// An iterator that holds a collected VecDeque of records in memory.
///
/// Have to do this because of an incompatibility between the Storage trait and
/// Rust smart pointers.
///
/// Not very optimized as all items need to be collected in memory, but should
/// be fine for our use case as CosmWasm contracts typically don't iterate more
/// than a few tens or at most a few hundreds records at a time.
///
/// This being said, we need to think about malicious contracts intentially
/// iterate a huge amount of data to cause memory issues.
pub(crate) struct MemIter {
    items: VecDeque<Record>,
}

impl MemIter {
    pub fn new(iter: impl Iterator<Item = Record>) -> Self {
        Self {
            items: iter.collect(),
        }
    }
}

impl Iterator for MemIter {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop_front()
    }
}

pub(crate) fn range_bounds(start: Option<&[u8]>, end: Option<&[u8]>) -> impl RangeBounds<Vec<u8>> {
    (
        start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
        end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
    )
}
