use sha2::{Digest, Sha256};

/// Perform a SHA-256 hash of the given bytes
pub fn sha256(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

/// Perform a SHA-256 hash, and truncate the output to the specified length.
pub fn sha256_truncate(bytes: &[u8], length: usize) -> Vec<u8> {
    let mut hash_bytes = sha256(bytes);
    hash_bytes.truncate(length);
    hash_bytes
}
