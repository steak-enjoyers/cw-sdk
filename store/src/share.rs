use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::{Order, Record, Storage};

use crate::iterators::MemIter;

/// Wrap a store in a smart pointer, so that it can be shared as an owned value.
/// Note that this struct can only be used in single threads.
///
/// Adapted from Orga:
/// https://github.com/nomic-io/orga/blob/v4/src/store/share.rs#L20
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T> Shared<T> {
    pub fn new(store: T) -> Self {
        Self(Rc::new(RefCell::new(store)))
    }

    pub fn share(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T: Storage> Storage for Shared<T> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.borrow().get(key)
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.0.borrow_mut().set(key, value)
    }

    fn remove(&mut self, key: &[u8]) {
        self.0.borrow_mut().remove(key)
    }

    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'a> {
        Box::new(MemIter::new(self.0.borrow().range(start, end, order)))
    }
}
