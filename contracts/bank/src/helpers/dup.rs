use std::{collections::BTreeSet, fmt::Display};

use thiserror::Error;

pub struct DupChecker<T> {
    /// A human-readable string describing the duplication of what is being checked
    ty: String,
    set: BTreeSet<T>,
}

impl<T> DupChecker<T>
where
    T: Ord + Display,
{
    /// Create a new instance of the duplicate checker
    pub fn new(ty: impl Into<String>) -> Self {
        Self {
            ty: ty.into(),
            set: BTreeSet::new(),
        }
    }

    /// If the value already exists in the set, throw a duplication error;
    /// otherwise, insert the value into the set.
    pub fn assert_no_dup(&mut self, value: T) -> Result<(), DuplicateError> {
        if self.set.contains(&value) {
            Err(DuplicateError {
                ty: self.ty.clone(),
                value: value.to_string(),
            })
        } else {
            self.set.insert(value);
            Ok(())
        }
    }
}

#[derive(Debug, Error)]
#[error("duplicate {ty}: {value}")]
pub struct DuplicateError {
    pub ty: String,
    pub value: String,
}
