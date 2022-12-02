use bech32::{FromBase32, ToBase32, Variant};
use cosmwasm_std::{Addr, CanonicalAddr};
use thiserror::Error;

use crate::hash::sha256;

/// Currently we simply hardcode the prefix in the state machine's binary.
///
/// Ideally, we prefer this to be a configurable value in the chain's state. However, this would
/// require a fork of cosmwasm-vm, which for now only support stateless address conversions:
/// https://github.com/CosmWasm/cosmwasm/blob/v1.1.4/packages/vm/src/backend.rs#L128-L129
pub const ADDRESS_PREFIX: &str = "cw";

/// The latest version of ADR-028 has increased the address length from 20 bytes to 32, due to
/// concerns of collisions.
///
/// References:
/// - ADR-028:
///   https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-028-public-key-addresses.md
/// - A related discussion on Ethereum forum:
///   https://ethereum-magicians.org/t/increasing-address-size-from-20-to-32-bytes/5485/43
pub const ADDRESS_LENGTH: usize = 32;

/// According to ADR-028, each basic address (one that is represented by a single key pair), needs
/// to have a "type" string denoting the public key scheme used.
///
/// For now, cw-sdk only supports the secp256k1 scheme. The type string is defined by:
/// https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/crypto/secp256k1/keys.proto
pub const PUBKEY_TYPE: &str = "cosmos.crypto.secp256k1.PubKey";

/// Takes a human readable address and returns a canonical binary representation of it.
pub fn canonicalize(human: &str) -> Result<CanonicalAddr, AddressError> {
    let (prefix, addr_bytes_base32, variant) = bech32::decode(human)?;

    let addr_bytes = Vec::<u8>::from_base32(&addr_bytes_base32)?;
    let addr_len = addr_bytes.len();

    // for now we only support bech32, not bech32m
    // but more research is needed regarding this choice
    if variant != Variant::Bech32 {
        Err(AddressError::IncorrectVariant)
    } else if prefix != ADDRESS_PREFIX {
        Err(AddressError::incorrect_prefix(prefix))
    } else if addr_len != ADDRESS_LENGTH {
        Err(AddressError::incorrect_length(addr_len))
    } else {
        Ok(addr_bytes.into())
    }
}

/// Takes a canonical address and returns a human readble address.
pub fn humanize(canonical: &CanonicalAddr) -> Result<Addr, AddressError> {
    let human = bech32::encode(ADDRESS_PREFIX, canonical.as_slice().to_base32(), Variant::Bech32)?;
    Ok(Addr::unchecked(human))
}

/// Takes a human readable address and validates if it is valid.
/// If it the validation succeeds, a `Addr` containing the same data as the input is returned.
pub fn validate(input: &str) -> Result<Addr, AddressError> {
    let canonical = canonicalize(input)?;
    let human = humanize(&canonical)?;
    if input == human {
        Ok(human)
    } else {
        Err(AddressError::recovered_mismatch(input, human))
    }
}

/// Derive an account address based on the public key.
///
/// The address bytes are computed as:
///
/// ```plain
/// address_bytes := sha256(PUBKEY_TYPE | sha256(pubkey_bytes))[:ADDRESS_LENGTH]
/// ```
///
/// Where `|` means bytes concatenation without using any separator.
pub fn derive_from_pubkey(pubkey_bytes: &[u8]) -> Result<Addr, AddressError> {
    let mut bytes = PUBKEY_TYPE.to_string().into_bytes();
    bytes.extend(sha256(pubkey_bytes));
    humanize_prehash(&bytes)
}

/// Derive contract address based on a human-readable label.
///
/// The address bytes are computed as:
///
/// ```plain
/// address_bytes := sha256("label" | label_bytes)[:ACCOUNT_LENGTH]
/// ```
///
/// Where `|` means bytes concatenation without using any separator.
pub fn derive_from_label(label: &str) -> Result<Addr, AddressError> {
    let mut bytes = "label".to_string().into_bytes();
    bytes.extend(label.to_string().into_bytes());
    humanize_prehash(&bytes)
}

/// Just a helper function for the `derive_from_*` methods.
/// Performs the last steps of the address derivation process according to ADR-028: take the hash,
/// truncate to the standard length, and humanize.
fn humanize_prehash(preimage_bytes: &[u8]) -> Result<Addr, AddressError> {
    let mut bytes = sha256(preimage_bytes);
    bytes.truncate(ADDRESS_LENGTH);
    humanize(&bytes.into())
}

#[derive(Debug, Error)]
pub enum AddressError {
    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error("incorrect address variant: expecting bech32, found bech32m")]
    IncorrectVariant,

    #[error("incorrect address prefix: expecting {expect}, found {found}")]
    IncorrectPrefix {
        expect: String,
        found: String,
    },

    #[error("incorrect address length: expecting {expect} bytes, found {found}")]
    IncorrectLength {
        expect: usize,
        found: usize,
    },

    #[error("address verification failed: input {input}, recovered {recovered}")]
    RecoveredMismatch {
        input: String,
        recovered: String,
    },
}

impl AddressError {
    pub fn incorrect_prefix(found: impl Into<String>) -> Self {
        Self::IncorrectPrefix {
            expect: ADDRESS_PREFIX.into(),
            found: found.into(),
        }
    }

    pub fn incorrect_length(found: usize) -> Self {
        Self::IncorrectLength {
            expect: ADDRESS_LENGTH,
            found,
        }
    }

    pub fn recovered_mismatch(input: impl Into<String>, recovered: impl Into<String>) -> Self {
        Self::RecoveredMismatch {
            input: input.into(),
            recovered: recovered.into(),
        }
    }
}
