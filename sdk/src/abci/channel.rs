use std::sync::mpsc::{Sender, Receiver};

use cosmwasm_std::Event;

use super::ABCIError;
use crate::msg::{CodeResponse, ContractResponse, WasmRawResponse, WasmSmartResponse, AccountResponse};

/// The ABCI server and the driver maintains a channel between them, and communicate by sending
/// commands.
///
/// This enum defines the commands allowed to be transmitted through the channel. They correspond
/// roughly one-to-one with elements in `SdkMsg` and `SdkQuery`.
///
/// TODO: probably should abstract so that this is independent of the specific state machine used
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Insert a wasm byte code into the app state, increment code count by 1.
    StoreCode {
        wasm_byte_code: Vec<u8>,
        /// Return the code id
        result_tx: Sender<u64>,
    },
    /// Instantiate a contract using the specified code id
    InstantiateContract {
        code_id: u64,
        msg: Vec<u8>,
        /// Return whether instantiation if successful, and if yes, the contract address
        result_tx: Sender<(bool, Option<u64>)>,
    },
    /// Execute a contract
    ExecuteContract {
        contract: u64,
        msg: Vec<u8>,
        result_tx: Sender<(bool, Option<Vec<Event>>)>,
    },
    QueryAccount {
        address: String,
        result_tx: Sender<AccountResponse>,
    },
    /// Query a wasm byte code based on code id
    QueryCode {
        code_id: u64,
        result_tx: Sender<CodeResponse>,
    },
    QueryContract {
        contract: u64,
        result_tx: Sender<ContractResponse>,
    },
    QueryWasmRaw {
        contract: u64,
        key: Vec<u8>,
        result_tx: Sender<WasmRawResponse>,
    },
    QueryWasmSmart {
        contract: u64,
        msg: Vec<u8>,
        result_tx: Sender<WasmSmartResponse>,
    },
}

/// A helper function for sending the specified value through a channel.
pub fn channel_send<T>(tx: &Sender<T>, value: T) -> Result<(), ABCIError> {
    tx.send(value).map_err(|_| ABCIError::Send)
}

/// A helper function for receiving data through a channel.
pub fn channel_recv<T>(rx: &Receiver<T>) -> Result<T, ABCIError> {
    rx.recv().map_err(ABCIError::from)
}
