use cosmwasm_std::Binary;
use secp256k1::{ecdsa::Signature, hashes::sha256::Hash as Sha256Hash, Message, PublicKey, Secp256k1};
use thiserror::Error;

use crate::address::Address;
use crate::msg::{Account, Tx};
use crate::State;

// TODO: include this in the chain's state
pub const ACCOUNT_PREFIX: &str = "cw";

/// Return the user's updated account info is authentication is successful; error if failed.
pub fn authenticate_tx(tx: &Tx, state: &State) -> Result<Account, AuthError> {
    // find the user's account
    let sender = &tx.body.sender;
    let mut account = match state.accounts.get(sender) {
        // if the account is found on-chain, its pubkey must match the one included in the tx
        Some(account) => {
            if let Some(pubkey) = &tx.pubkey {
                if account.pubkey != *pubkey {
                    return Err(AuthError::pubkey_mismatch(sender, &account.pubkey, pubkey));
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

            let address = Address::from_pubkey(pubkey.as_slice());
            let bech32_addr = address.bech32(ACCOUNT_PREFIX)?;
            if *sender != bech32_addr {
                return Err(AuthError::address_mismatch(bech32_addr, sender));
            }

            Account {
                pubkey: pubkey.clone(),
                sequence: 0,
            }
        },
    };

    // the chain id must match
    if state.chain_id != tx.body.chain_id {
        return Err(AuthError::chain_id_mismatch(&state.chain_id, &tx.body.chain_id));
    }

    // the account sequence mush match
    account.sequence += 1;
    if account.sequence != tx.body.sequence {
        return Err(AuthError::sequence_mismatch(sender, account.sequence, tx.body.sequence));
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
        Ok(()) => Ok(account),
        Err(err) => Err(err.into()),
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error(transparent)]
    Serialization(#[from] serde_json_wasm::ser::Error),

    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),

    #[error(transparent)]
    Bech32(#[from] bech32::Error),

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

impl AuthError {
    pub fn account_not_found(sender: impl Into<String>) -> Self {
        Self::AccountNotFound {
            sender: sender.into(),
        }
    }

    pub fn address_mismatch(expect: impl Into<String>, found: impl Into<String>) -> Self {
        Self::AddressMismatch {
            expect: expect.into(),
            found: found.into(),
        }
    }

    pub fn pubkey_mismatch(sender: impl Into<String>, expect: &Binary, found: &Binary) -> Self {
        Self::PubkeyMismatch {
            sender: sender.into(),
            expect: hex::encode(expect.as_slice()),
            found: hex::encode(found.as_slice()),
        }
    }

    pub fn chain_id_mismatch(expect: impl Into<String>, found: impl Into<String>) -> Self {
        Self::ChainIdMismatch {
            expect: expect.into(),
            found: found.into(),
        }
    }

    pub fn sequence_mismatch(sender: impl Into<String>, expect: u64, found: u64) -> Self {
        Self::SequenceMismatch {
            sender: sender.into(),
            expect,
            found,
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
