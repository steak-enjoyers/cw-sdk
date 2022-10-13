use sha2::{Digest, Sha256};

/// Perform a SHA-256 hash of the given bytes
pub fn sha256(bytes: impl AsRef<[u8]>) -> Vec<u8> 
{
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}
