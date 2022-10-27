use cosmwasm_std::{Addr, Binary};
use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use thiserror::Error;

use cw_sdk::address::{self, AddressError};
use cw_sdk::{Account, Tx};

use crate::state::State;

/// Authenticate the signer's address, pubkey, signature, sequence, and chain id.
/// Return error if any one fails.
/// Returns the sender address and account info if succeeds.
pub fn authenticate_tx(tx: &Tx, state: &State) -> Result<Sender, AuthError> {
    let sender = &tx.body.sender;
    let sender_addr = address::validate(sender)?;

    // find the user's account
    let (pubkey, mut sequence) = match state.accounts.get(&sender_addr) {
        // If the sender account is a contract, throw error because contracts can't sign txs.
        Some(Account::Contract {
            ..
        }) => {
            return Err(AuthError::AccountIsContract);
        }

        // If the account is found on chain, meaning the account has already sent at least one tx
        // before, its pubkey must match the one included in the tx.
        Some(Account::Base {
            pubkey,
            sequence,
        }) => {
            if let Some(sender_pubkey) = &tx.pubkey {
                if pubkey != sender_pubkey {
                    return Err(AuthError::pubkey_mismatch(sender, pubkey, sender_pubkey));
                }
            }

            (pubkey.clone(), *sequence)
        },

        // If not found, meaning it's the first time the account every sends a tx, use the pubkey
        // provided by the tx and initialize sequence to be 0.
        // Note, the pubkey must match the sender address.
        None => {
            let pubkey = tx.pubkey.as_ref().ok_or_else(|| AuthError::account_not_found(sender))?;

            let address = address::derive_from_pubkey(pubkey.as_slice())?;
            if *sender != address {
                return Err(AuthError::address_mismatch(address, sender));
            }

            (pubkey.clone(), 0)
        },
    };

    // the chain id must match
    if state.chain_id != tx.body.chain_id {
        return Err(AuthError::chain_id_mismatch(&state.chain_id, &tx.body.chain_id));
    }

    // the account sequence mush match
    sequence += 1;
    if sequence != tx.body.sequence {
        return Err(AuthError::sequence_mismatch(sender, sequence, tx.body.sequence));
    }

    // verify the signature
    // the content to be signed is (the sha256 hash of) the tx body
    let body_bytes = serde_json::to_vec(&tx.body)?;
    let signature = Signature::try_from(tx.signature.as_slice())?;

    // if signature is valid, return the sender address and updated account info
    // otherwise, return error
    VerifyingKey::from_sec1_bytes(pubkey.as_slice())?
        .verify(&body_bytes, &signature)
        .map(|_| Sender {
            address: sender_addr,
            account: Account::Base {
                pubkey,
                sequence,
            },
        })
        .map_err(AuthError::from)
}

pub struct Sender {
    pub address: Addr,
    pub account: Account<Addr>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Ecdsa(#[from] k256::ecdsa::Error),

    #[error(transparent)]
    Address(#[from] AddressError),

    #[error("sender account is a contract")]
    AccountIsContract,

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
