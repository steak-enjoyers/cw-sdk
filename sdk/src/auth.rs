use bech32::{ToBase32, Variant};
use secp256k1::{ecdsa::Signature, hashes::sha256::Hash as Sha256Hash, Message, PublicKey, Secp256k1};
use thiserror::Error;

use crate::msg::{Account, Tx};
use crate::State;

// TODO: include this in the chain's state
const ACCOUNT_PREFIX: &str = "cw";

/// NOTE: we take a `&mut State` because if the tx is authenticated, we need to update the user's
/// pubkey (if previously not recorded) and sequence number. maybe think of a way to separate the
/// state-mutating portion of the function and non-mutating part into separate functions.
pub fn authenticate_tx(tx: &Tx, state: &mut State) -> Result<bool, AuthError> {
    // find the user's account
    let sender = &tx.body.sender;
    let mut account = match state.get_account(sender) {
        // if the account is found on-chain, its pubkey must match the one included in the tx
        Some(account) => {
            if let Some(pubkey) = &tx.pubkey {
                if account.pubkey != *pubkey {
                    return Err(AuthError::PubkeyMismatch {
                        sender: sender.into(),
                        expect: hex::encode(account.pubkey.as_slice()),
                        found: hex::encode(pubkey.as_slice()),
                    });
                }
            }
            account.clone()
        },
        // if None, use the pubkey provided by the tx and initialize sequence to be 0.
        // the pubkey must match the sender address.
        None => {
            let pubkey = tx.pubkey.as_ref().ok_or_else(|| AuthError::AccountNotFound {
                sender: sender.into(),
            })?;

            let address = adr28_addr(ACCOUNT_PREFIX, pubkey.as_slice())?;
            if sender != &address {
                return Err(AuthError::AddressMismatch {
                    expect: address,
                    found: sender.into(),
                });
            }

            Account {
                pubkey: pubkey.clone(),
                sequence: 0,
            }
        },
    };

    // the chain id must match
    let chain_id = state.get_chain_id();
    if chain_id != tx.body.chain_id {
        return Err(AuthError::ChainIdMismatch {
            expect: chain_id.into(),
            found: tx.body.chain_id.clone(),
        });
    }

    // the account sequence mush match
    account.sequence += 1;
    if account.sequence != tx.body.sequence {
        return Err(AuthError::SequenceMismatch {
            sender: sender.into(),
            expect: account.sequence,
            found: tx.body.sequence,
        });
    }

    // verify the signature. the content to be signed is (the sha256 hash of) the tx body
    let body_bytes = serde_json_wasm::to_vec(&tx.body)?;

    // if the signature is valid, save the account, and return true; otherwise, return false
    //
    // this part of code is mostly copied from:
    // https://github.com/nomic-io/orga/blob/dc864db8a6e42723afd26d1dea9245bb620fa488/src/plugins/signer.rs#L117-L141
    let pubkey = PublicKey::from_slice(account.pubkey.as_slice())?;
    let message = Message::from_hashed_data::<Sha256Hash>(&body_bytes);
    let signature = Signature::from_compact(&tx.signature)?;
    match Secp256k1::new().verify_ecdsa(&message, &signature, &pubkey) {
        Ok(()) => {
            // TODO: only update account if it's DeliverTx; don't update if it's CheckTx
            state.set_account(sender, account);
            Ok(true)
        },
        Err(err) => Err(AuthError::from(err)),
    }
}

/// Convert a pubkey to bech32 address according to
/// [ADR-028](https://docs.cosmos.network/master/architecture/adr-028-public-key-addresses.html),
/// which specifies that address bytes are to be computed as:
/// ```plain
/// address_bytes := ripemd160(sha256(pubkey_bytes))[:20]
/// ```
pub fn adr28_addr(prefix: &str, pubkey: &[u8]) -> Result<String, AuthError> {
    use ripemd::Ripemd160;
    use sha2::{Digest, Sha256};

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

    let account_bytes = ripemd160(&sha256(pubkey));
    bech32::encode(prefix, account_bytes.to_base32(), Variant::Bech32).map_err(AuthError::from)
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("failed to serialize tx body into json: {0}")]
    Serialization(#[from] serde_json_wasm::ser::Error),

    #[error("failed to encode pubkey into bech32: {0}")]
    Bech32(#[from] bech32::Error),

    #[error("error while validating secp256k1 signature: {0}")]
    Secp256k1(#[from] secp256k1::Error),

    #[error("pubkey for sender {sender} is neither provided in the tx nor stored on-chain")]
    AccountNotFound {
        sender: String,
    },

    #[error("sender address does not match pubkey: expecting {expect}, found {found}")]
    AddressMismatch {
        // The sender address deduced from the provided pubkey
        expect: String,
        // The sender address actually provided by the tx
        found: String,
    },

    #[error("pubkey for sender {sender} does not match: expecting {expect}, found {found}")]
    PubkeyMismatch {
        sender: String,
        /// The pubkey stored on-chain; hex-encoded bytearray
        expect: String,
        /// The pubkey included in the tx; hex-encoded bytearray
        found: String,
    },

    #[error("incorrect chain id: expecting {expect}, found {found}")]
    ChainIdMismatch {
        /// The chain id stored on-chain
        expect: String,
        /// The chain id provided by the tx
        found: String,
    },

    #[error("incorrect sequence number for sender {sender}: expecting {expect}, found {found}")]
    SequenceMismatch {
        sender: String,
        /// The sequence number stored on-chain plus 1
        expect: u64,
        /// The sequence number provided by the tx
        found: u64,
    },
}

#[cfg(test)]
mod tests {
    // TODO
}
