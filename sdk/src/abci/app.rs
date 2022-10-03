use std::sync::mpsc::{channel, Sender};

use tendermint_proto::abci::{
    Event, EventAttribute, RequestDeliverTx, RequestQuery, ResponseDeliverTx, ResponseQuery,
};

use super::{channel_recv, channel_send, ABCIError, AppCommand};
use crate::{SdkMsg, SdkQuery};

#[derive(Clone, Debug)]
pub struct App {
    pub cmd_tx: Sender<AppCommand>,
}

impl App {
    pub fn store_code(&self, wasm_byte_code: Vec<u8>) -> Result<ResponseDeliverTx, ABCIError> {
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

    pub fn query_code(&self, code_id: u64) -> Result<ResponseQuery, ABCIError> {
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
