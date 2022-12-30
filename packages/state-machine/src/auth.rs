use cosmwasm_std::{Addr, BlockInfo, Storage};
use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

use cw_sdk::{address, Account, Tx};

use crate::{
    error::{Error, Result},
    state::ACCOUNTS,
};

/// The response type of `authenticate_tx` function.
pub struct Sender {
    pub address: Addr,
    pub account: Account<Addr>,
}

/// Authenticate the signer's address, pubkey, signature, sequence, and chain id.
/// Return error if any one fails.
/// Returns the sender address and account info if succeeds.
pub fn authenticate_tx(store: &dyn Storage, pending_block: &BlockInfo, tx: &Tx) -> Result<Sender> {
    let sender = &tx.body.sender;
    let sender_addr = address::validate(sender)?;

    // find the user's account
    let (pubkey, mut sequence) = match ACCOUNTS.may_load(store, &sender_addr)? {
        // If the sender account is a contract, throw error because contracts
        // can't sign txs.
        Some(Account::Contract {
            ..
        }) => {
            return Err(Error::account_is_contract(sender));
        }

        // If the account is found on chain, meaning the account has already
        // sent at least one tx before, its pubkey must match the one included
        // in the tx.
        Some(Account::Base {
            pubkey,
            sequence,
        }) => {
            if let Some(sender_pubkey) = &tx.pubkey {
                if pubkey != *sender_pubkey {
                    return Err(Error::pubkey_mismatch(sender, &pubkey, sender_pubkey));
                }
            }

            (pubkey, sequence)
        },

        // If not found, meaning it's the first time the account every sends a
        // tx, use the pubkey provided by the tx and initialize sequence to be 0.
        // Note, the pubkey must match the sender address.
        None => {
            let Some(pubkey) = &tx.pubkey else {
                return Err(Error::account_not_found(sender));
            };

            let address = address::derive_from_pubkey(pubkey.as_slice())?;
            if *sender != address {
                return Err(Error::address_mismatch(address, sender));
            }

            (pubkey.clone(), 0)
        },
    };

    // the chain id must match
    if pending_block.chain_id != tx.body.chain_id {
        return Err(Error::chain_id_mismatch(&pending_block.chain_id, &tx.body.chain_id));
    }

    // the account sequence mush match
    sequence += 1;
    if sequence != tx.body.sequence {
        return Err(Error::sequence_mismatch(sender, sequence, tx.body.sequence));
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
        .map_err(Error::from)
}
