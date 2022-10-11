use std::sync::mpsc::Sender;

use cosmwasm_std::Event;

use crate::StateError;

/// The ABCI server and the driver maintains a channel between them, and communicate by sending
/// commands. This enum defines the commands allowed to be transmitted through the channel.
#[derive(Debug, Clone)]
pub enum AppCommand {
    Query {
        query_bytes: Vec<u8>,
        result_tx: Sender<Result<Vec<u8>, StateError>>,
    },
    DeliverTx {
        tx_bytes: Vec<u8>,
        result_tx: Sender<Result<Vec<Event>, StateError>>,
    }
}
