use bip32::{Mnemonic, XPrv};
use cosmwasm_std::Addr;
use josekit::jwt::JwtPayload;
use k256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};

use cw_sdk::address;
use cw_sdk::msg::{Tx, TxBody};

use crate::DaemonError;

/// Represents a private key that is to be saved in the keyring.
///
/// The format used here is different from that used by Go SDK
/// (https://github.com/cosmos/keyring/blob/v1.2.0/keyring.go#L76-L86), but backward compatibility
/// is not a primary concern for me now.
#[derive(Debug, Clone)]
pub struct Key {
    /// The key's name
    pub name: String,
    /// The private key
    sk: SigningKey,
}

impl Key {
    /// Create a new key instance from a given name and BIP-32 mnemonic phrase
    pub fn from_mnemonic(
        name: impl Into<String>,
        mnemonic: &Mnemonic,
        coin_type: u32,
    ) -> Result<Self, DaemonError> {
        // The `to_seed` function takes a password to generate salt. Here we just use an empty str.
        // For reference, both Terra Station and Keplr use an empty string as well:
        // - https://github.com/terra-money/terra.js/blob/v3.1.7/src/key/MnemonicKey.ts#L79
        // - https://github.com/chainapsis/keplr-wallet/blob/b6062a4d24f3dcb15dda063b1ece7d1fbffdbfc8/packages/crypto/src/mnemonic.ts#L63
        let seed = mnemonic.to_seed("");
        let path = format!("m/44'/{}'/0'/0/0", coin_type);
        let xprv = XPrv::derive_from_path(&seed, &path.parse()?)?;
        Ok(Self {
            name: name.into(),
            sk: xprv.into(),
        })
    }

    /// Create a new key instance from a given name and private key bytes
    pub fn from_privkey_bytes(name: impl Into<String>, sk_bytes: &[u8]) -> Result<Self, DaemonError> {
        let sk = SigningKey::from_bytes(sk_bytes)?;
        Ok(Self {
            name: name.into(),
            sk,
        })
    }

    /// Return a reference to the private key
    pub fn privkey(&self) -> &SigningKey {
        &self.sk
    }

    /// Return the pubkey
    pub fn pubkey(&self) -> VerifyingKey {
        self.sk.verifying_key()
    }

    /// Return the key's address bytes, generated according to
    /// [ADR-028](https://docs.cosmos.network/v0.45/architecture/adr-028-public-key-addresses.html)
    pub fn address(&self) -> Result<Addr, address::AddressError> {
        address::derive_from_pubkey(self.pubkey().to_bytes().as_slice())
    }

    /// Sign an arbitrary byte array. The bytes are SHA-256 hashed before signing
    pub fn sign_bytes(&self, bytes: &[u8]) -> Signature {
        self.sk.sign(bytes)
    }

    /// Sign a tx body, returns the full tx.
    pub fn sign_tx(&self, body: &TxBody) -> Result<Tx, DaemonError> {
        let body_bytes = serde_json::to_vec(body)?;
        let signature = self.sign_bytes(&body_bytes);
        Ok(Tx {
            body: body.clone(),
            pubkey: Some(self.pubkey().to_bytes().to_vec().into()),
            signature: signature.to_vec().into(),
        })
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
    type Error = DaemonError;

    fn try_from(payload: JwtPayload) -> Result<Self, Self::Error> {
        let name = payload
            .claim("name")
            .ok_or_else(|| DaemonError::malformed_payload("key `name` not found"))?
            .as_str()
            .ok_or_else(|| DaemonError::malformed_payload("incorrect JSON value type for `name`"))?;
        let sk_str = payload
            .claim("sk")
            .ok_or_else(|| DaemonError::malformed_payload("key `sk` not found"))?
            .as_str()
            .ok_or_else(|| DaemonError::malformed_payload("incorrect JSON value type for `sk`"))?;

        let sk_bytes = hex::decode(sk_str)?;
        Key::from_privkey_bytes(name, &sk_bytes)
    }
}
