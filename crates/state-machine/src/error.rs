#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Std(#[from] cosmwasm_std::StdError),

    #[error(transparent)]
    Backend(#[from] cosmwasm_vm::BackendError),

    #[error(transparent)]
    Vm(#[from] cosmwasm_vm::VmError),

    #[error(transparent)]
    Merk(#[from] cw_store::MerkError),

    #[error(transparent)]
    Address(#[from] cw_sdk::address::AddressError),

    #[error(transparent)]
    Ecdsa(#[from] k256::ecdsa::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("contract emitted error: {0}")]
    Contract(String),

    #[error("an account already exists with the address {address}")]
    AccountFound {
        address: String,
    },

    #[error("no account found with the address {address}")]
    AccountNotFound {
        address: String,
    },

    #[error("the account associated with the address {address} is a contract")]
    AccountIsContract {
        address: String,
    },

    #[error("the account associated with the address {address} is not a contract")]
    AccountIsNotContract {
        address: String,
    },

    #[error("no wasm binary code found with id {code_id}")]
    CodeNotFound {
        code_id: u64,
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

    #[error("contract response includes submessages, which is not supported yet")]
    SubmessagesUnsupported,

    #[error("sending funds when instantiating or executing contracts is not supported yet")]
    FundsUnsupported,

    #[error("migrating contracts is not supported yet")]
    MigrationUnsupported,

    #[error("this query is not supported yet")]
    QueryUnsupported,
}

impl Error {
    pub fn account_found(address: impl Into<String>) -> Self {
        Self::AccountFound {
            address: address.into(),
        }
    }

    pub fn account_not_found(address: impl Into<String>) -> Self {
        Self::AccountNotFound {
            address: address.into(),
        }
    }

    pub fn account_is_contract(address: impl Into<String>) -> Self {
        Self::AccountIsContract {
            address: address.into(),
        }
    }

    pub fn account_is_not_contract(address: impl Into<String>) -> Self {
        Self::AccountIsNotContract {
            address: address.into(),
        }
    }

    pub fn code_not_found(code_id: u64) -> Self {
        Self::CodeNotFound {
            code_id,
        }
    }

    pub fn address_mismatch(expect: impl Into<String>, found: impl Into<String>) -> Self {
        Self::AddressMismatch {
            expect: expect.into(),
            found: found.into(),
        }
    }

    pub fn pubkey_mismatch(sender: impl Into<String>, expect: &[u8], found: &[u8]) -> Self {
        Self::PubkeyMismatch {
            sender: sender.into(),
            expect: hex::encode(expect),
            found: hex::encode(found),
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

pub type Result<T> = core::result::Result<T, Error>;
