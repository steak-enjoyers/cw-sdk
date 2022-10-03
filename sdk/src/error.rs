use std::sync::mpsc::RecvError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error{"channel send error"}]
    Send,

    #[error("channel receive error: {0}")]
    Recv(#[from] RecvError),
}
