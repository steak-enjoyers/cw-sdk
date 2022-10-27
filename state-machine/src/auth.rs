use cosmwasm_std::{Addr, Binary};
use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use thiserror::Error;

use cw_types::address::{self, AddressError};
use cw_types::msg::Tx;

use crate::state::{Account, State};

/// Authenticate the signer's address, pubkey, signature, sequence, and chain id.
/// Return error if any one fails.
/// Returns the sender address and account info if succeeds.
pub fn authenticate_tx(tx: &Tx, state: &State) -> Result<(Addr, Account), AuthError> {
    let sender = &tx.body.sender;
    let sender_addr = address::validate(&tx.body.sender)?;

    // find the user's account
    let mut account = match state.accounts.get(&sender_addr) {
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
            let pubkey = tx.pubkey.as_ref().ok_or_else(|| AuthError::account_not_found(sender))?;

            let address = address::derive_from_pubkey(pubkey.as_slice())?;
            if *sender != address {
                return Err(AuthError::address_mismatch(address, sender));
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
    let body_bytes = serde_json::to_vec(&tx.body)?;
    let pubkey = VerifyingKey::from_sec1_bytes(account.pubkey.as_slice())?;
    let signature = Signature::try_from(tx.signature.as_slice())?;

    // if signature is valid, return the sender address and updated account info
    // otherwise, return error
    pubkey.verify(&body_bytes, &signature)
        .map(|_| (sender_addr, account))
        .map_err(AuthError::from)
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Ecdsa(#[from] k256::ecdsa::Error),

    #[error(transparent)]
    Address(#[from] AddressError),

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
