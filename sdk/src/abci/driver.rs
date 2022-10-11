use std::sync::mpsc::Receiver;

use super::{channel_send, AppCommand, ABCIError};
use crate::State;

/// The driver is a wrapper around the actual state machine. It maintains a channel with the ABCI
/// server, and performs actions or queries on the state machine on request for the ABCI server.
pub struct AppDriver {
    pub state: State,
    pub cmd_rx: Receiver<AppCommand>,
}

impl AppDriver {
    pub fn run(mut self) -> Result<(), ABCIError> {
        loop {
            match self.cmd_rx.recv()? {
                AppCommand::StoreCode {
                    wasm_byte_code,
                    result_tx,
                } => {
                    let code_id = self.state.store_code(wasm_byte_code)?;
                    channel_send(&result_tx, code_id)?;
                },
                AppCommand::InstantiateContract {
                    code_id,
                    msg,
                    result_tx,
                } => {
                    let (success, contract) = self.state.instantiate_contract(code_id, msg)?;
                    channel_send(&result_tx, (success, contract))?;
                },
                AppCommand::ExecuteContract {
                    contract,
                    msg,
                    result_tx,
                } => {
                    let (success, events) = self.state.execute_contract(contract, msg)?;
                    channel_send(&result_tx, (success, events))?;
                },
                AppCommand::QueryAccount {
                    address,
                    result_tx,
                } => {
                    let response = self.state.query_account(&address)?;
                    channel_send(&result_tx, response)?;
                }
                AppCommand::QueryCode {
                    code_id,
                    result_tx,
                } => {
                    let response = self.state.query_code(code_id)?;
                    channel_send(&result_tx, response)?;
                },
                AppCommand::QueryContract {
                    contract,
                    result_tx,
                } => {
                    let response = self.state.query_contract(contract)?;
                    channel_send(&result_tx, response)?;
                },
                AppCommand::QueryWasmRaw {
                    contract,
                    key,
                    result_tx,
                } => {
                    let response = self.state.query_wasm_raw(contract, &key)?;
                    channel_send(&result_tx, response)?;
                },
                AppCommand::QueryWasmSmart {
                    contract,
                    msg,
                    result_tx,
                } => {
                    let response = self.state.query_wasm_smart(contract, &msg)?;
                    channel_send(&result_tx, response)?;
                }
            }
        }
    }
}
