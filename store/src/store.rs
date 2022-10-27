use cosmwasm_std::Storage;
use merk::{BatchEntry, Merk, Op};
use std::{
    mem::transmute,
    path::{Path, PathBuf},
};

use crate::cache::MemoryStorage;

pub struct Store {
    merk: Merk,
    home: PathBuf,
    cache: MemoryStorage,
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
            cache,
        }
    }

    /// Utility fns shamelessly pinched from orga
    fn path<T: ToString>(&self, name: T) -> PathBuf {
        self.home.join(name.to_string())
    }

    /// Again, very similar to the orga implementation,
    /// but modified to work on the signature of MemoryStorage;
    /// we also omit the aux_batch for now
    pub(super) fn write(&mut self) -> Result<()> {
        // note that memory_storage is just a mapping
        let memory_storage = self.cache.data.take().unwrap();

        // re-init the MemoryStorage
        self.cache = Some(MemoryStorage::new());

        let batch = to_batch(memory_storage);

        Ok(self
            .merk
            .as_mut()
            .unwrap()
            .apply(batch.as_ref())?)
    }

    pub(super) fn merk(&self) -> &Merk {
        self.merk.as_ref().unwrap()
    }
}

/// This collects a k/v iterator into a Vec
/// which can then be used with merk.apply
/// this is adapted from orga's state layer
pub fn to_batch<I: IntoIterator<Item = (Vec<u8>, <Vec<u8>>)>>(i: I) -> Vec<BatchEntry> {
    let mut batch = Vec::new();
    for (key, val) in i {
        match val {
            Some(val) => batch.push((key, Op::Put(val))),
            None => batch.push((key, Op::Delete)),
        }
    }
    batch
}

impl Storage for Store {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.merk.as_ref().unwrap().get(key)
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.merk.as_mut().unwrap().insert(key, Some(value))
    }

    fn remove(&mut self, key: &[u8]) {
        self.merk.as_mut().unwrap().insert(key.to_vec(), None)
    }

    // #[cfg(feature = "iterator")]
    // fn range<'a>(
    //       &'a self,
    //       start: Option<&[u8]>,
    //       end: Option<&[u8]>,
    //       order: Order,
    //   ) -> Box<dyn Iterator<Item = Record> + 'a> {

    // }
}
