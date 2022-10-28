use cosmwasm_std::Storage;
use merk::{Error as MerkError, Merk, Op};

use std::path::PathBuf;

use crate::cache::{Delta, LocalState};

pub struct Store {
    merk: Merk,
    home: PathBuf,
}

/// First create a basic store
/// this implementation closely mirrors
/// what nomic do in orga
impl Store {
    pub fn new(home: PathBuf) -> Self {
        let merk = Merk::open(&home.join("db")).unwrap();

        Store {
            home,
            merk,
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
    /// NB BatchEntry == (Vec<u8>, Op)
    /// aux_batch is for meta, static config etc
    /// NB apply_unchecked because (Mappum):
    /// "Since you know all the keys are sorted and unique,
    /// you can... skip the check in apply"
    pub fn write(
        &mut self,
        pending_deltas: LocalState,
        aux_batch: Vec<(Vec<u8>, Op)>,
    ) -> Result<(), MerkError> {
        let pending_batch: Vec<(Vec<u8>, Op)> = deltas_to_ops(pending_deltas);
        Ok(self.merk.apply_unchecked(&pending_batch, &aux_batch)?)
    }

    /// Flush to underlying store
    /// this removes second argument so it fits the more generic interface
    /// in transaction where the only args are Storage and Deltas
    pub fn write_deltas(&mut self, pending_deltas: LocalState) -> Result<(), MerkError> {
        let pending_batch: Vec<(Vec<u8>, Op)> = deltas_to_ops(pending_deltas);
        Ok(self.merk.apply_unchecked(&pending_batch, &[])?)
    }

    /// Flush a collection of ops to store
    /// we can't make assumptions, therefore apply, not apply_unchecked
    pub fn commit(
        &mut self,
        pending_batch: Vec<(Vec<u8>, Op)>,
        aux_batch: Vec<(Vec<u8>, Op)>,
    ) -> Result<(), MerkError> {
        Ok(self.merk.apply(&pending_batch, &aux_batch)?)
    }
}

/// There's a subtle difference between merk Ops
/// and the cw-storage style Ops
/// this is a helper to account for that
/// we could define on Delta, but better for each storage
/// to just handle it with a match in its own scope
fn deltas_to_ops(deltas: LocalState) -> Vec<(Vec<u8>, Op)> {
    deltas.into_iter().map(|(key, delta)| (key, delta_to_op(delta, key))).collect()
}

fn delta_to_op(delta: Delta, key: Vec<u8>) -> Op {
    match delta {
        Delta::Set {
            value,
        } => Op::Put(value.to_vec()),
        Delta::Delete {} => Op::Delete {},
    }
}

impl Storage for Store {
    /// Go direct to underlying merk
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.merk.get(key) {
            Ok(res) => res,
            Err(_) => None,
        }
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.merk.apply(&[(key.to_vec(), Op::Put(value.to_vec()))], &[]);
    }

    fn remove(&mut self, key: &[u8]) {
        self.merk.apply(&[(key.to_vec(), Op::Delete)], &[]);
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
