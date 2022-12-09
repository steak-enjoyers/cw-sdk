use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

use crate::SdkMsg;

/// Tendermint will provide this as JSON bytes by in the CheckTx and DeliverTx
/// requests. The state machine should deserialize the bytes upon receipt.
#[cw_serde]
pub struct Tx {
    /// Transaction body, which includes the sender address, messages to be
    /// executed in order, and some parameters for prevention of replay attacks.
    pub body: TxBody,

    /// The sender's secp256k1 public key.
    /// Optional if the accounts already exists in the state.
    pub pubkey: Option<Binary>,

    /// Secp256k1 signature.
    /// The content is `sha256(JSON.stringify(txbody))`, signed by the
    /// corresponding private key.
    pub signature: Binary,
}

/// Body of the transaction. This is what the sender needs to sign.
#[cw_serde]
pub struct TxBody {
    /// The sender's address
    pub sender: String,

    /// Identifier of the chain where this tx is to be broadcasted.
    /// Used to prevent reply attacks.
    pub chain_id: String,

    /// The sender's sequence number.
    /// Used to prvent replay attacks.
    pub sequence: u64,

    /// Wasm messages to be executed in order
    pub msgs: Vec<SdkMsg>,
}
