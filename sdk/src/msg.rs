use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin};

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
pub enum SdkQuery {
    Code {
        code_id: u64,
    },
    Contract {
        contract: u64,
    },
    WasmRaw {
        contract: u64,
        key: Binary,
    },
    WasmSmart {
        contract: u64,
        msg: Binary,
    },
}
