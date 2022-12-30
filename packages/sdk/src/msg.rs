use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, BlockInfo, Coin, ContractResult};
use serde_json::Value;

use crate::account::Account;

#[cw_serde]
pub enum SdkMsg {
    /// Store a binary code to the blockchain's state.
    StoreCode {
        wasm_byte_code: Binary,
    },

    /// Instantiate a new contract account.
    Instantiate {
        /// Identifier of the wasm byte code to be associated with the contract
        code_id: u64,

        /// JSON-encoded instantiate message
        msg: Value,

        /// Coins to be sent to the contract during instantiation
        funds: Vec<Coin>,

        /// A human readable name for the contract. Must be unique.
        //
        /// Contracts addresses derived deterministically from the label, using
        /// the same algorithm that the Go SDK generates module account addresses.
        ///
        /// There are several special labels, such as `bank`, `staking`, `gov`,
        /// `ibc`, etc., that developers need to pay special attention to.
        ///
        /// For example,
        /// - the SDK invokes the "bank" contract to process gas fee payments
        /// - IBC relayers will invoke the "ibc" contract to deliver packets
        ///
        /// For such labels, developers must make sure to deploy contracts that
        /// have compatible execute/query/sudo methods implemented.
        label: String,

        /// Account who is allowed to migrate the contract.
        /// To make the contract immutable, leave this field empty.
        admin: Option<String>,
    },

    /// Execute a contract
    Execute {
        contract: String,
        msg: Value,
        funds: Vec<Coin>,
    },

    /// Migrate a contract to a new wasm byte code
    Migrate {
        contract: String,
        code_id: u64,
        msg: Value,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum SdkQuery {
    /// Query the state machine's overall info, such as block height, chain id, etc.
    #[returns(InfoResponse)]
    Info {},

    /// Query a single account by address
    #[returns(AccountResponse)]
    Account {
        address: String,
    },

    /// Enumerate all accounts by address
    #[returns(Vec<AccountResponse>)]
    Accounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Query a single contract by label
    #[returns(ContractResponse)]
    Contract {
        label: String,
    },

    /// Enumerate all contracts by label
    #[returns(Vec<ContractResponse>)]
    Contracts {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Query a single wasm byte code by id
    #[returns(CodeResponse)]
    Code {
        code_id: u64,
    },

    /// Enumerate all wasm byte codes by code id
    #[returns(Vec<CodeResponse>)]
    Codes {
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    /// Perform raw query on a wasm contract
    #[returns(WasmRawResponse)]
    WasmRaw {
        contract: String,
        key: Binary,
    },

    /// Perform smart query on a wasm contract
    #[returns(WasmSmartResponse)]
    WasmSmart {
        contract: String,
        msg: Value,
    },
}

#[cw_serde]
pub struct InfoResponse {
    pub last_committed_block: BlockInfo,
    pub code_count: u64,
}

#[cw_serde]
pub struct AccountResponse {
    pub address: String,
    pub account: Account<String>,
}

#[cw_serde]
pub struct ContractResponse {
    pub address: String,
    pub code_id: u64,
    pub label: String,
    pub admin: Option<String>,
}

#[cw_serde]
pub struct CodeResponse {
    pub code_id: u64,
    pub wasm_byte_code: Binary,
}

#[cw_serde]
pub struct WasmRawResponse {
    /// Raw value in the contract storage under the given key.
    /// None if the key is not found.
    pub value: Option<Binary>,
}

#[cw_serde]
pub struct WasmSmartResponse {
    /// Smart query result.
    /// The querying program is responsible for decoding the binary response
    /// into the correct type.
    pub result: ContractResult<Binary>,
}
