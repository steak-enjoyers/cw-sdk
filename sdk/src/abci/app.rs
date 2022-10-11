use std::sync::mpsc::{channel, Sender};

use cosmwasm_std::{Attribute as WasmAttribute, Event as WasmEvent};
use tendermint_proto::abci::{
    Event, EventAttribute, RequestDeliverTx, RequestQuery, ResponseDeliverTx, ResponseQuery,
};

use super::AppCommand;

#[derive(Clone, Debug)]
pub struct App {
    pub cmd_tx: Sender<AppCommand>,
}

impl tendermint_abci::Application for App {
    fn query(&self, request: RequestQuery) -> ResponseQuery {
        let path = request.path.split("/").collect::<Vec<_>>();

        if path.is_empty() {
            return ResponseQuery {
                code: 1,
                log: "no query path provided".into(),
                ..Default::default()
            };
        }

        match path[0] {
            "app" => {
                let (result_tx, result_rx) = channel();

                self.cmd_tx
                    .send(AppCommand::Query {
                        query_bytes: request.data,
                        result_tx,
                    })
                    .unwrap();
                let result = result_rx.recv().unwrap();

                match result {
                    Ok(response_bytes) => ResponseQuery {
                        code: 0,
                        value: response_bytes,
                        ..Default::default()
                    },
                    Err(error) => ResponseQuery {
                        code: 1,
                        log: error.to_string(),
                        ..Default::default()
                    },
                }
            },
            "store" => {
                // unimplemented
                ResponseQuery {
                    code: 1,
                    log: "store query is not implemented yet".into(),
                    ..Default::default()
                }
            },
            "p2p" => {
                // unimplemented as well
                // however, return no error to signal that the peer should not be rejected
                // see:
                // https://github.com/tendermint/tendermint/blob/v0.34.x/spec/abci/apps.md#query-connection
                ResponseQuery {
                    code: 0,
                    log: "p2p query is not implemented yet".into(),
                    ..Default::default()
                }
            },
        }
    }

    fn deliver_tx(&self, request: RequestDeliverTx) -> ResponseDeliverTx {
        let (result_tx, result_rx) = channel();

        self.cmd_tx
            .send(AppCommand::DeliverTx {
                tx_bytes: request.tx,
                result_tx,
            })
            .unwrap();
        let result = result_rx.recv().unwrap();

        match result {
            // TODO: what should we put in `data` and `log` fields?
            // for now i just serialize the events into a JSON string as log
            Ok(events) => ResponseDeliverTx {
                code: 0,
                log: serde_json_wasm::to_string(&events).unwrap(),
                events: wasm_event_to_abci(events),
                ..Default::default()
            },
            Err(error) => ResponseDeliverTx {
                code: 1,
                log: error.to_string(),
                ..Default::default()
            },
        }
    }
}

/// Casting CosmWasm event attributes into ABCI event attributes
fn wasm_attrs_to_abci(wasm_attrs: Vec<WasmAttribute>) -> Vec<EventAttribute> {
    wasm_attrs
        .into_iter()
        .map(|attr| EventAttribute {
            key: attr.key.into_bytes(),
            value: attr.key.into_bytes(),
            // Not sure what "index" means, but Go SDK returns `true` for all attributes,
            // so I'll do the same here =)
            index: true,
        })
        .collect()
}

/// Casting CosmWasm events into ABCI events
fn wasm_event_to_abci(wasm_events: Vec<WasmEvent>) -> Vec<Event> {
    wasm_events
        .into_iter()
        .map(|event| Event {
            r#type: event.ty,
            attributes: wasm_attrs_to_abci(event.attributes),
        })
        .collect()
}
