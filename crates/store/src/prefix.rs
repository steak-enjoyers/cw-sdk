//! Contents of this file are adapted from cw-storage-plus:
//! https://github.com/CosmWasm/cw-multi-test/blob/v0.16.0/src/prefixed_storage.rs

pub fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut c = a.to_vec();
    c.extend_from_slice(b);
    c
}

pub fn trim(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    key[namespace.len()..].to_vec()
}

/// Returns a new vec of same length and last byte incremented by one
/// If last bytes are 255, we handle overflow up the chain.
/// If all bytes are 255, this returns wrong data - but that is never possible as a namespace
pub fn namespace_upper_bound(input: &[u8]) -> Vec<u8> {
    let mut copy = input.to_vec();
    // zero out all trailing 255, increment first that is not such
    for i in (0..input.len()).rev() {
        if copy[i] == 255 {
            copy[i] = 0;
        } else {
            copy[i] += 1;
            break;
        }
    }
    copy
}
