use bech32::{ToBase32, Variant};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use thiserror::Error;

pub struct Address(Vec<u8>);

impl Address {
    /// Convert a pubkey bytes to address bytes according to
    /// [ADR-028](https://docs.cosmos.network/master/architecture/adr-028-public-key-addresses.html)
    pub fn from_pubkey(pubkey_bytes: &[u8]) -> Self {
        let address_bytes = ripemd160(&sha256(pubkey_bytes));
        Self(address_bytes)
    }

    /// Return a reference of the address bytes
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// Return hex encoding of the address
    pub fn hex(&self) -> String {
        hex::encode(self.bytes())
    }

    /// Return the bech32 encoding of the address with the given prefix
    pub fn bech32(&self, prefix: &str) -> Result<String, AddressError> {
        bech32::encode(prefix, self.bytes().to_base32(), Variant::Bech32)
            .map_err(AddressError::from)
    }
}

fn sha256(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

fn ripemd160(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

#[derive(Debug, Error)]
pub enum AddressError {
    #[error("failed to encode address into bech32: {0}")]
    Bech32(#[from] bech32::Error),
}
