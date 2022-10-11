use std::sync::mpsc::Receiver;

use super::AppCommand;
use crate::State;

/// The driver is a wrapper around the actual state machine. It maintains a channel with the ABCI
/// server, and performs actions or queries on the state machine on request for the ABCI server.
pub struct AppDriver {
    pub state: State,
    pub cmd_rx: Receiver<AppCommand>,
}

impl AppDriver {
    pub fn run(mut self) {
        loop {
            match self.cmd_rx.recv().unwrap() {
                AppCommand::Query {
                    query_bytes,
                    result_tx,
                } => result_tx.send(self.state.handle_query(&query_bytes)).unwrap(),
                AppCommand::DeliverTx {
                    tx_bytes,
                    result_tx,
                } => result_tx.send(self.state.handle_tx(&tx_bytes)).unwrap(),
            }
        }
    }
}
