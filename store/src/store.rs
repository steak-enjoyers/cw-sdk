#[cfg(feature = "iterator")]
use core::ops::{Bound, RangeBounds};
use cosmwasm_std::{Order, Record, Storage};
use merk::{Error as MerkError, Merk, Op};
#[cfg(feature = "iterator")]
use std::iter;
use std::{collections::BTreeMap, path::PathBuf};

type PendingOps = BTreeMap<Vec<u8>, Op>;

pub struct Store {
    merk: Merk,
    home: PathBuf,
    pending: PendingOps,
}

/// First create a basic store
/// this implementation closely mirrors
/// what nomic do in orga
impl Store {
    pub fn new(home: PathBuf) -> Self {
        let merk = Merk::open(&home.join("db")).unwrap();
        let pending = PendingOps::new();

        Store {
            home,
            merk,
            pending,
        }
    }

    /// Utility fns shamelessly pinched from orga
    fn path<T: ToString>(&self, name: T) -> PathBuf {
        self.home.join(name.to_string())
    }

    pub(super) fn merk(&self) -> &Merk {
        &self.merk
    }

    /// Flush to underlying store
    /// our pending vec should already be the same
    /// as BatchEntry, i.e. (Vec<u8>, Op)
    /// aux_batch is for meta, static config etc
    pub fn write(&mut self, aux_batch: Vec<(Vec<u8>, Op)>) -> Result<(), MerkError> {
        // first convert BTreeMap -> Vec
        let pending_batch: Vec<(Vec<u8>, Op)> =
            self.pending.into_iter().map(|(key, op)| (key, op)).collect();

        // send to merk
        self.merk.apply(&pending_batch, &aux_batch)?;

        // reset pending
        self.pending = PendingOps::new();

        Ok(())
    }
}

impl Storage for Store {
    /// First try pending
    /// then fall back to merk
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.pending.get(key) {
            Some(Op::Put(value)) => Some(value.clone()),
            Some(Op::Delete) => None,
            None => match self.merk.get(key) {
                Ok(res) => res,
                Err(_) => None,
            },
        }
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.pending.insert(key.to_vec(), Op::Put(value.to_vec()));
    }

    fn remove(&mut self, key: &[u8]) {
        self.pending.insert(key.to_vec(), Op::Delete);
    }

    // #[cfg(feature = "iterator")]
    // /// range allows iteration over a set of keys, either forwards or backwards
    // /// uses standard rust range notation, and eg db.range(b"foo"..b"bar") also works reverse
    // fn range<'a>(
    //     &'a self,
    //     start: Option<&[u8]>,
    //     end: Option<&[u8]>,
    //     order: Order,
    // ) -> Box<dyn Iterator<Item = Record> + 'a> {
    //     let bounds = range_bounds(start, end);

    //     // BTreeMap.range panics if range is start > end.
    //     // However, this cases represent just empty range and we treat it as such.
    //     match (bounds.start_bound(), bounds.end_bound()) {
    //         (Bound::Included(start), Bound::Excluded(end)) if start > end => {
    //             return Box::new(iter::empty());
    //         },
    //         _ => {},
    //     }

    //     let iter = self.pending.range(bounds);
    //     match order {
    //         Order::Ascending => Box::new(iter.map(clone_item)),
    //         Order::Descending => Box::new(iter.rev().map(clone_item)),
    //     }
    // }
}

// #[cfg(feature = "iterator")]
// fn range_bounds(start: Option<&[u8]>, end: Option<&[u8]>) -> impl RangeBounds<Vec<u8>> {
//     (
//         start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
//         end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
//     )
// }

// #[cfg(feature = "iterator")]
// /// The BTreeMap specific key-value pair reference type, as returned by BTreeMap<Vec<u8>, Vec<u8>>::range.
// /// This is internal as it can change any time if the map implementation is swapped out.
// type BTreeMapRecordRef<'a> = (&'a Vec<u8>, &'a Vec<u8>);

// #[cfg(feature = "iterator")]
// fn clone_item(item_ref: BTreeMapRecordRef) -> Record {
//     let (key, value) = item_ref;
//     (key.clone(), value.clone())
// }
