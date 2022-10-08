use bip32::{Mnemonic, XPrv};
use josekit::jwt::JwtPayload;
use k256::ecdsa::{SigningKey, VerifyingKey};
use thiserror::Error;

use cw_sdk::address::Address;

/// Represents a private key that can be used to sign stuff.
///
/// The format used here is different from that used by Go SDK
/// (https://github.com/cosmos/keyring/blob/v1.2.0/keyring.go#L76-L86), but backward compatibility
/// is not a primary concern for me now.
#[derive(Debug, Clone)]
pub struct Key {
    /// The key's name
    pub name: String,
    /// The private key
    pub sk: SigningKey,
}

impl Key {
    /// Create a new key instance from a given name and BIP-32 mnemonic phrase
    pub fn from_mnemonic(
        name: impl Into<String>,
        mnemonic: &Mnemonic,
        coin_type: u32,
    ) -> Result<Self, KeyError> {
        // The `to_seed` function takes a password to generate salt. Here we just use an empty
        // string. My current knowledge is not sufficient to determine whether this is safe or not.
        // For reference:
        // - Terra Station uses empty string:
        //   https://github.com/terra-money/terra.js/blob/v3.1.7/src/key/MnemonicKey.ts#L79
        // - Keplr uses an emtpy string by default. Haven't looked up what it uses in practice yet:
        //   https://github.com/chainapsis/keplr-wallet/blob/b6062a4d24f3dcb15dda063b1ece7d1fbffdbfc8/packages/crypto/src/mnemonic.ts#L63
        let seed = mnemonic.to_seed("");
        let path = format!("m/44'/{}'/0'/0/0", coin_type);
        let xprv = XPrv::derive_from_path(&seed, &path.parse()?)?;
        Ok(Self {
            name: name.into(),
            sk: xprv.into(),
        })
    }

    /// Create a new key instance from a given name and private key bytes
    pub fn from_bytes(name: impl Into<String>, sk_bytes: &[u8]) -> Result<Self, KeyError> {
        let sk = SigningKey::from_bytes(sk_bytes)?;
        Ok(Self {
            name: name.into(),
            sk,
        })
    }

    /// Return the private key bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.sk.to_bytes().to_vec()
    }

    /// Return the key's pubkey
    pub fn pubkey(&self) -> VerifyingKey {
        self.sk.verifying_key()
    }

    /// Return the key's address bytes, generated according to
    /// [ADR-028](https://docs.cosmos.network/v0.45/architecture/adr-028-public-key-addresses.html)
    pub fn address(&self) -> Address {
        Address::from_pubkey(self.pubkey().to_bytes().as_slice())
    }
}

impl TryFrom<Key> for JwtPayload {
    type Error = josekit::JoseError;

    fn try_from(key: Key) -> Result<Self, Self::Error> {
        let sk_str = hex::encode(key.sk.to_bytes().as_slice());
        let mut payload = JwtPayload::new();
        payload.set_claim("name", Some(key.name.into()))?;
        payload.set_claim("sk", Some(sk_str.into()))?;
        Ok(payload)
    }
}

impl TryFrom<JwtPayload> for Key {
    type Error = KeyError;

    fn try_from(payload: JwtPayload) -> Result<Self, Self::Error> {
        let name = payload
            .claim("name")
            .ok_or_else(|| KeyError::malformed_payload("key `name` not found"))?
            .as_str()
            .ok_or_else(|| KeyError::malformed_payload("incorrect JSON value type for `name`"))?;
        let sk_str = payload
            .claim("sk")
            .ok_or_else(|| KeyError::malformed_payload("key `sk` not found"))?
            .as_str()
            .ok_or_else(|| KeyError::malformed_payload("incorrect JSON value type for `sk`"))?;

        let sk_bytes = hex::decode(sk_str)?;
        Key::from_bytes(name, &sk_bytes)
    }
}

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("{0}")]
    Bip32(#[from] bip32::Error),

    #[error("{0}")]
    Ecdsa(#[from] k256::ecdsa::Error),

    #[error("{0}")]
    FromHex(#[from] hex::FromHexError),

    #[error("failed to cast JWT payload to key: {reason}")]
    MalformedPayload {
        reason: String,
    },
}

impl KeyError {
    pub fn malformed_payload(reason: impl Into<String>) -> Self {
        Self::MalformedPayload {
            reason: reason.into(),
        }
    }
}
