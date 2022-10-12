use std::str::FromStr;

use bech32::{FromBase32, ToBase32, Variant};
use thiserror::Error;

use crate::hash::{sha256, sha256_truncate};

/// The latest version of [ADR-028](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-028-public-key-addresses.md)
/// has increated the address length from 20 bytes to 32, due to concerns of collisions.
pub const ADDRESS_LENGTH: usize = 32;

/// According to ADR-028, each basic address (one that is represented by a single key pair), need to
/// have a "type" string denoting its public key schema used.
///
/// For now, cw-sdk only supports the secp256k1 public key. The type string is defined by:
/// https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/crypto/secp256k1/keys.proto
pub const ACCOUNT_TYPE: &str = "cosmos.crypto.secp256k1.PubKey";

/// Represents an account address
pub struct Address(Vec<u8>);

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, addr_bytes_base32, variant) = bech32::decode(s)?;
        let addr_bytes = Vec::<u8>::from_base32(&addr_bytes_base32)?;
        // for now we only support bech32, not bech32m
        // but more research is needed regarding this choice
        if variant != Variant::Bech32 {
            Err(AddressError::IncorrectVariant)
        } else if addr_bytes.len() != ADDRESS_LENGTH {
            Err(AddressError::incorrect_length(addr_bytes.len()))
        } else {
            Ok(Self(addr_bytes))
        }
    }
}

impl Address {
    /// Convert a pubkey bytes to address bytes according to
    /// [ADR-028](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-028-public-key-addresses.md).
    ///
    /// The address bytes are derived as
    ///
    /// ```plain
    /// address_bytes := sha256(ACCOUNT_TYPE | sha256(pubkey_bytes))[:ADDRESS_LENGTH]
    /// ```
    ///
    /// Where `|` means bytes concatenation without using any separator.
    pub fn from_pubkey(pubkey_bytes: &[u8]) -> Self {
        let mut bytes = ACCOUNT_TYPE.to_string().into_bytes();
        bytes.extend(sha256(pubkey_bytes));
        Self(sha256_truncate(&bytes, ADDRESS_LENGTH))
    }

    /// Derive contract address based on the contract's label. This is used when instantiating
    /// contracts during genesis.
    ///
    /// The address bytes are derived as
    ///
    /// ```plain
    /// address_bytes := sha256("label" | label_bytes)[:ACCOUNT_LENGTH]
    /// ```
    ///
    /// Where `|` means bytes concatenation without using any separator.
    pub fn from_label(label: &str) -> Self {
        let mut bytes = "label".to_string().into_bytes();
        bytes.extend(label.to_string().into_bytes());
        Self(sha256_truncate(&bytes, ADDRESS_LENGTH))
    }

    /// Derive contract address based on the code id and instance id. This is used when
    /// instantiating contracts post-genesis.
    ///
    /// The address bytes are derived as
    ///
    /// ```plain
    /// address_bytes := sha256("ids" | code_id_be_bytes | instance_id_be_bytes)[:ACCOUNT_LENGTH]
    /// ```
    ///
    /// Where `|` means bytes concatenation without using any separator, and `*_be_bytes` big endian
    /// bytes of a number. Here, both `code_id` and `instance_id` are 64-bit unsigned integers, so
    /// their lengths should be 8 bytes each.
    pub fn from_ids(code_id: u64, instance_id: u64) -> Self {
        let mut bytes = "ids".to_string().into_bytes();
        bytes.extend(code_id.to_be_bytes());
        bytes.extend(instance_id.to_be_bytes());
        Self(sha256_truncate(&bytes, ADDRESS_LENGTH))
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

#[derive(Debug, Error)]
pub enum AddressError {
    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error("incorrect address variant: expecting bech32, found bech32m")]
    IncorrectVariant,

    #[error("incorrect address length: expecting {expect} bytes, found {found}")]
    IncorrectLength {
        expect: usize,
        found: usize,
    },
}

impl AddressError {
    pub fn incorrect_length(found: usize) -> Self {
        Self::IncorrectLength {
            expect: ADDRESS_LENGTH,
            found,
        }
    }
}