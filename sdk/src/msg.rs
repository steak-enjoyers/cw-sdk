use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, ContractResult};

/// This should be included as JSON inside `~/.tendermint/genesis.json`, under the `app_state`
/// field. Tendermint will provide this as binary to the application in a InitChain request.
#[derive(Default)]
#[cw_serde]
pub struct GenesisState {
    /// Address of the account which will act as the sender of genesis messages.
    ///
    /// For example, if an "Instantiate" message in included in `gen_msgs`, then the deployer
    /// address will be provided as `info.sender` in the instantiation call.
    ///
    /// Note that during genesis, no transaction verification is performed. The application
    /// developers must provide a trust deployer account.
    pub deployer: String,
    /// Messages to be executed in order during the InitChain call.
    pub gen_msgs: Vec<SdkMsg>,
}

#[cw_serde]
pub struct Tx {
    /// Transaction body
    pub body: TxBody,
    /// The sender's secp256k1 public key. Optional if the accounts already exists in the state.
    pub pubkey: Option<Binary>,
    /// Secp256k1 signature; the content is `sha256(JSON.stringify(txbody))`, signed by the private
    /// key corresponding to `pubkey`.
    pub signature: Binary,
}

/// Body of the transaction. This is what the sender needs to sign.
#[cw_serde]
pub struct TxBody {
    /// The sender's address
    pub sender: String,
    /// Wasm messages to be executed
    pub msgs: Vec<SdkMsg>,
    /// Identifier of the chain where this tx is to be broadcasted. Used to prevent reply attacks.
    pub chain_id: String,
    /// The sender's sequence number. Used to prvent replay attacks.
    pub sequence: u64,
}

#[cw_serde]
pub enum SdkMsg {
    StoreCode {
        wasm_byte_code: Binary,
    },
    Instantiate {
        code_id: u64,
        msg: Binary,
        funds: Vec<Coin>,
        /// A human readable name for the contract. Must be unique.
        ///
        /// Contracts deployed during genesis will have their addresses generated deterministically
        /// according to the label, using the same algorithm that the Go SDK generates module
        /// account addresses.
        ///
        /// There are several special labels, such as `bank`, `staking`, `gov`, `ibc`, etc., that
        /// developers need to pay special attention to. For example,
        ///
        /// - the SDK invokes the "bank" contract to process gas fee payments
        /// - IBC relayers will invoke the "ibc" contract to deliver packets
        ///
        /// For such labels, developers must make sure to deploy contracts that have compatible
        /// execute/query/sudo methods implemented.
        label: String,
    },
    Execute {
        contract: u64,
        msg: Binary,
        funds: Vec<Coin>,
    },
    Migrate {
        contract: u64,
        code_id: u64,
        msg: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum SdkQuery {
    #[returns(AccountResponse)]
    Account {
        address: String,
    },
    #[returns(CodeResponse)]
    Code {
        code_id: u64,
    },
    #[returns(ContractResponse)]
    Contract {
        contract: u64,
    },
    #[returns(WasmRawResponse)]
    WasmRaw {
        contract: u64,
        key: Binary,
    },
    #[returns(WasmSmartResponse)]
    WasmSmart {
        contract: u64,
        msg: Binary,
    },
}

/// This is the account type to be stored on-chain. Not to be confused with `AccountResponse`.
#[cw_serde]
pub struct Account {
    /// The account's secp256k1 public key
    pub pubkey: Binary,
    /// The account's sequence number, used to prevent replay attacks.
    /// The first tx ever to be submitted by the account should come with the sequence of 1.
    pub sequence: u64,
}

#[cw_serde]
pub struct AccountResponse {
    pub address: String,
    /// None is the account is not found
    pub pubkey: Option<Binary>,
    /// Zero if account is not found
    pub sequence: u64,
}

#[cw_serde]
pub struct CodeResponse {
    /// SHA-256 hash of the wasm byte code
    pub hash: Binary,
    /// The wasm byte code
    pub wasm_byte_code: Binary,
}

#[cw_serde]
pub struct ContractResponse {
    /// This contract's code id
    pub code_id: u64,
}

#[cw_serde]
pub struct WasmRawResponse {
    /// Contract address
    pub contract: u64,
    /// The key
    pub key: Binary,
    /// Raw value in the contract storage under the given key. None if the key does not exist.
    pub value: Option<Binary>,
}

#[cw_serde]
pub struct WasmSmartResponse {
    /// Contract address
    pub contract: u64,
    /// Smart query result.
    /// The querying program is responsible for decoding the binary response into the correct type.
    pub result: ContractResult<Binary>,
}
