// The `Digest` trait can be imported from either ripemd or sha2. It's the same.
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

pub fn sha256(bytes: impl AsRef<[u8]>) -> Vec<u8> 
{
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub fn ripemd160(bytes: impl AsRef<[u8]>) -> Vec<u8> 
{
    let mut hasher = Ripemd160::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}
