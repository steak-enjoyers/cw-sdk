use std::sync::mpsc::{channel, Receiver, Sender};

use tendermint_proto::abci::{
    Event, EventAttribute, RequestDeliverTx, RequestQuery, ResponseDeliverTx, ResponseQuery,
};

use crate::{AppError, AppState, SdkMsg, SdkQuery};

#[derive(Clone, Debug)]
pub struct App {
    pub cmd_tx: Sender<AppCommand>,
}

impl App {
    pub fn store_code(&self, wasm_byte_code: Vec<u8>) -> Result<ResponseDeliverTx, AppError> {
        let (result_tx, result_rx) = channel();

        channel_send(
            &self.cmd_tx,
            AppCommand::StoreCode {
                wasm_byte_code,
                result_tx,
            },
        )?;
        let code_id = channel_recv(&result_rx)?;

        Ok(ResponseDeliverTx {
            code: 0,
            data: vec![],
            log: "successfully stored code!".to_string(),
            info: "".to_string(),
            gas_wanted: 0,
            gas_used: 0,
            events: vec![Event {
                r#type: "store_code".to_string(),
                attributes: vec![EventAttribute {
                    key: "code_id".as_bytes().to_owned(),
                    value: code_id.to_string().into_bytes(),
                    index: false,
                }],
            }],
            codespace: "".to_string(),
        })
    }

    pub fn query_code(&self, code_id: u64) -> Result<ResponseQuery, AppError> {
        let (result_tx, result_rx) = channel();

        channel_send(
            &self.cmd_tx,
            AppCommand::QueryCode {
                code_id,
                result_tx,
            },
        )?;
        let wasm_byte_code = channel_recv(&result_rx)?;

        let log = if wasm_byte_code.is_some() {
            "exists"
        } else {
            "does not exist"
        };

        Ok(ResponseQuery {
            code: 0,
            log: log.to_string(),
            info: "".to_string(),
            index: 0,
            key: code_id.to_string().into_bytes(),
            value: wasm_byte_code.unwrap_or_default(),
            proof_ops: None,
            height: 0,
            codespace: "".to_string(),
        })
    }
}

impl tendermint_abci::Application for App {
    fn query(&self, request: RequestQuery) -> ResponseQuery {
        let query = match serde_json_wasm::from_slice::<SdkQuery>(&request.data) {
            Ok(query) => query,
            Err(err) => {
                return error_query(
                    format!("Error: failed to unmarshal query: {}", err),
                    request.data,
                )
            },
        };

        match query {
            SdkQuery::Code {
                code_id,
            } => self.query_code(code_id).unwrap(),
        }
    }

    fn deliver_tx(&self, request: RequestDeliverTx) -> ResponseDeliverTx {
        let msg = match serde_json_wasm::from_slice::<SdkMsg>(&request.tx) {
            Ok(msg) => msg,
            Err(err) => {
                return error_deliver_tx(format!("Error: failed to unmarshal message: {}", err))
            },
        };

        match msg {
            SdkMsg::StoreCode {
                wasm_byte_code,
            } => self.store_code(wasm_byte_code.0).unwrap(),
            msg => error_deliver_tx(format!("Error: unimplemented sdk message: {:?}", msg)),
        }
    }
}

fn error_query(log: impl ToString, key: Vec<u8>) -> ResponseQuery {
    ResponseQuery {
        code: 0,
        log: log.to_string(),
        info: "".to_string(),
        index: 0,
        key,
        value: vec![],
        proof_ops: None,
        height: 0,
        codespace: "".to_string(),
    }
}

fn error_deliver_tx(log: impl ToString) -> ResponseDeliverTx {
    ResponseDeliverTx {
        code: 0,
        data: vec![],
        log: log.to_string(),
        info: "".to_string(),
        gas_wanted: 0,
        gas_used: 0,
        events: vec![],
        codespace: "".to_string(),
    }
}

#[derive(Debug)]
pub struct AppDriver {
    pub state: AppState,
    pub cmd_rx: Receiver<AppCommand>,
}

impl AppDriver {
    pub fn run(mut self) -> Result<(), AppError> {
        loop {
            match self.cmd_rx.recv()? {
                AppCommand::StoreCode {
                    wasm_byte_code,
                    result_tx,
                } => {
                    self.state.code_count += 1;
                    let code_id = self.state.code_count;
                    // no need to check whether a code already exists under this code id
                    self.state.codes.insert(code_id, wasm_byte_code);
                    channel_send(&result_tx, code_id)?;
                },
                AppCommand::QueryCode {
                    code_id,
                    result_tx,
                } => {
                    let wasm_byte_code = self.state.codes.get(&code_id);
                    channel_send(&result_tx, wasm_byte_code.cloned())?;
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Insert a wasm byte code into the app state, increment code count by 1; return the code id
    StoreCode {
        wasm_byte_code: Vec<u8>,
        result_tx: Sender<u64>,
    },
    /// Query a code based on code id; return Some(wasm_byte_code) if code exists, or None if not
    QueryCode {
        code_id: u64,
        result_tx: Sender<Option<Vec<u8>>>,
    },
}

fn channel_send<T>(tx: &Sender<T>, value: T) -> Result<(), AppError> {
    tx.send(value).map_err(|_| AppError::Send)
}

fn channel_recv<T>(rx: &Receiver<T>) -> Result<T, AppError> {
    rx.recv().map_err(AppError::from)
}
