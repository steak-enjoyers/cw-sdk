// The `Digest` trait can be imported from either ripemd or sha2. It's the same.
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

/// Is there any way to let this take both `&Vec<u8>` and `&[u8]`?
///
/// Currently, to hash an Option<&Vec<u8>>, I have to to
///
/// ```no_run
/// let bytes: Option<&Vec<u8>>;
/// let hash = bytes.map(|bytes| sha256(bytes));
/// ```
///
/// However I would prefer to simply do
///
/// ```no_run
/// let hash = bytes.map(sha256);
/// ```
///
/// Maybe try the AsRef trait, idk
pub fn sha256(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub fn ripemd160(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}
