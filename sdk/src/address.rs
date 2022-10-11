use std::str::FromStr;

use bech32::{FromBase32, ToBase32, Variant};

use crate::hash::{ripemd160, sha256};

/// Represents an account address
pub struct Address(Vec<u8>);

impl FromStr for Address {
    type Err = bech32::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, addr_bytes_base32, _) = bech32::decode(s)?;
        let addr_bytes = Vec::<u8>::from_base32(&addr_bytes_base32)?;
        Ok(Self(addr_bytes))
    }
}

impl Address {
    /// Convert a pubkey bytes to address bytes according to
    /// [ADR-028](https://docs.cosmos.network/master/architecture/adr-028-public-key-addresses.html)
    pub fn from_pubkey(pubkey_bytes: &[u8]) -> Self {
        let addr_bytes = ripemd160(&sha256(pubkey_bytes));
        Self(addr_bytes)
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
    pub fn bech32(&self, prefix: &str) -> Result<String, bech32::Error> {
        bech32::encode(prefix, self.bytes().to_base32(), Variant::Bech32)
    }
}
