use std::sync::mpsc::RecvError;

use thiserror::Error;

use crate::StateError;

#[derive(Debug, Error)]
pub enum ABCIError {
    #[error{"channel send error"}]
    Send,

    #[error("channel receive error: {0}")]
    Recv(#[from] RecvError),

    #[error("error while executing or querying the state machine: {0}")]
    State(#[from] StateError),
}
