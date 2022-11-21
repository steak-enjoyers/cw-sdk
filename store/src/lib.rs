#![feature(btree_drain_filter)]

mod cache;
mod helpers;
mod iterators;
mod prefix;
mod store;

pub use crate::cache::CachedStore;
pub use crate::iterators::{MemIter, MergedIter, MerkIter};
pub use crate::prefix::PrefixedStore;
pub use crate::store::{PendingStoreWrapper, SharedStore, Store, StoreWrapper};

pub use merk::Error as MerkError;
