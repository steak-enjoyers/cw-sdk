use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary};
use cw_address_like::AddressLike;

/// The account type to be stored on-chain.
#[cw_serde]
pub enum Account<T: AddressLike> {
    /// An account that is controlled by a single public/private key pair.
    /// Roughly synonymous to "externally-owned account" (EoA) in Ethereum.
    Base {
        /// The account's secp256k1 public key
        pubkey: Binary,

        /// The account's sequence number, used to prevent replay attacks.
        /// The first tx ever to be submitted by the account should come with the sequence of 1.
        sequence: u64,
    },

    /// An account that is controlled by wasm code.
    Contract {
        /// Identifier of the wasm byte code associated with this contract.
        code_id: u64,

        /// A human readable name for the contract
        label: String,

        /// Account who is allowed to migrate the contract
        admin: Option<T>,
    },
}

impl From<Account<Addr>> for Account<String> {
    fn from(acct: Account<Addr>) -> Self {
        match acct {
            Account::Base {
                pubkey,
                sequence,
            } => Account::Base {
                pubkey,
                sequence,
            },
            Account::Contract {
                code_id,
                label,
                admin,
            } => Account::Contract {
                code_id,
                label,
                admin: admin.map(String::from),
            },
        }
    }
}
