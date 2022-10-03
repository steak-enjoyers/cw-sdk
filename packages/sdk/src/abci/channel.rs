use std::sync::mpsc::{Sender, Receiver};

use super::ABCIError;

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
    /// Query a code based on code id
    QueryCode {
        code_id: u64,
        /// Return `Some(wasm_byte_code)` if code exists, `None` otherwise
        result_tx: Sender<Option<Vec<u8>>>,
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
