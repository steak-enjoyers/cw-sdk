use std::sync::mpsc::{channel, Sender};

use cosmwasm_std::{Attribute as WasmAttribute, BlockInfo, Event as WasmEvent, Timestamp};
use cw_sdk::{GenesisState, SdkQuery, Tx};
use tendermint_proto::abci::{self, Event, EventAttribute};

use crate::AppCommand;

#[derive(Clone, Debug)]
pub struct App {
    pub cmd_tx: Sender<AppCommand>,
}

impl tendermint_abci::Application for App {
    /// Provide information about the ABCI application.
    ///
    /// TODO: `abci::Requestinfo` has three parameters: version, block_version,
    /// and p2p_version. I don't know what they mean or how to handle them.
    /// For now they are just ignored.
    fn info(&self, _request: abci::RequestInfo) -> abci::ResponseInfo {
        let (result_tx, result_rx) = channel();

        self.cmd_tx
            .send(AppCommand::Info {
                result_tx,
            })
            .unwrap();
        let (height, app_hash) = result_rx.recv().unwrap().unwrap();

        abci::ResponseInfo {
            data: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
            app_version: 1,
            last_block_height: height,
            last_block_app_hash: app_hash.to_vec(),
        }
    }

    /// Called once upon genesis.
    fn init_chain(&self, request: abci::RequestInitChain) -> abci::ResponseInitChain {
        let (result_tx, result_rx) = channel();

        let gen_state: GenesisState = serde_json::from_slice(&request.app_state_bytes).unwrap_or_else(|err| {
            panic!("failed to parse genesis state: {err}");
        });

        self.cmd_tx
            .send(AppCommand::InitChain {
                chain_id: request.chain_id,
                gen_state,
                result_tx,
            })
            .unwrap();
        let app_hash = result_rx.recv().unwrap().unwrap();

        abci::ResponseInitChain {
            app_hash: app_hash.to_vec(),
            ..Default::default()
        }
    }

    /// Query the application for data at the current or past height.
    fn query(&self, request: abci::RequestQuery) -> abci::ResponseQuery {
        let path = request.path.split('/').collect::<Vec<_>>();

        if path.is_empty() {
            return abci::ResponseQuery {
                code: 1,
                log: "no query path provided".into(),
                ..Default::default()
            };
        }

        match &path[0] {
            &"app" => {
                let (result_tx, result_rx) = channel();

                let query: SdkQuery = serde_json::from_slice(&request.data).unwrap_or_else(|err| {
                    panic!("failed to deserialize query message: {err}");
                });

                self.cmd_tx
                    .send(AppCommand::Query {
                        query,
                        result_tx,
                    })
                    .unwrap();
                let result = result_rx.recv().unwrap();

                match result {
                    Ok(response) => abci::ResponseQuery {
                        code: 0,
                        value: response.to_vec(),
                        ..Default::default()
                    },
                    Err(error) => abci::ResponseQuery {
                        // TODO: we need to define error codes instead of using
                        // `1` for all errors
                        code: 1,
                        log: error.to_string(),
                        ..Default::default()
                    },
                }
            },
            &"store" => {
                // unimplemented
                abci::ResponseQuery {
                    code: 1,
                    log: "store query is not implemented yet".into(),
                    ..Default::default()
                }
            },
            &"p2p" => {
                // unimplemented as well
                // however, return no error to signal that the peer should not be rejected
                // see:
                // https://github.com/tendermint/tendermint/blob/v0.34.x/spec/abci/apps.md#query-connection
                abci::ResponseQuery {
                    code: 0,
                    log: "p2p query is not implemented yet".into(),
                    ..Default::default()
                }
            },
            prefix => abci::ResponseQuery {
                code: 1,
                log: format!("unsupported query path prefix: {prefix}"),
                ..Default::default()
            },
        }
    }

    /// Check the given transaction before putting it into the local mempool.
    fn check_tx(&self, _request: abci::RequestCheckTx) -> abci::ResponseCheckTx {
        Default::default()
    }

    /// Signals the beginning of a new block, prior to any `DeliverTx` calls.
    fn begin_block(&self, request: abci::RequestBeginBlock) -> abci::ResponseBeginBlock {
        let (result_tx, result_rx) = channel();

        let header = request.header.unwrap();

        let protobuf_time = header.time.unwrap();
        let time = Timestamp::from_nanos(
            u64::try_from(protobuf_time.seconds).unwrap() * 10u64.pow(9) +
            u64::try_from(protobuf_time.nanos).unwrap(),
        );

        let block = BlockInfo {
            height: header.height as u64,
            time: time,
            chain_id: header.chain_id,
        };

        self.cmd_tx
            .send(AppCommand::BeginBlock {
                block,
                result_tx,
            })
            .unwrap();
        let result = result_rx.recv().unwrap();

        abci::ResponseBeginBlock {
            events: wasm_event_to_abci(result.unwrap()),
        }
    }

    /// Apply a transaction to the application's state.
    fn deliver_tx(&self, request: abci::RequestDeliverTx) -> abci::ResponseDeliverTx {
        let (result_tx, result_rx) = channel();

        let tx: Tx = serde_json::from_slice(&request.tx).unwrap_or_else(|err| {
            panic!("failed to deserialize tx: {err}");
        });

        self.cmd_tx
            .send(AppCommand::DeliverTx {
                tx,
                result_tx,
            })
            .unwrap();
        let result = result_rx.recv().unwrap();

        match result {
            // TODO: what should we put in `data` and `log` fields?
            // for now i just serialize the events into a JSON string as log
            Ok(events) => abci::ResponseDeliverTx {
                code: 0,
                log: serde_json::to_string(&events).unwrap(),
                events: wasm_event_to_abci(events),
                ..Default::default()
            },
            Err(error) => abci::ResponseDeliverTx {
                code: 1,
                log: error.to_string(),
                ..Default::default()
            },
        }
    }

    /// Signals the end of a block.
    fn end_block(&self, _request: abci::RequestEndBlock) -> abci::ResponseEndBlock {
        Default::default()
    }

    /// Commit the current state at the current height.
    fn commit(&self) -> abci::ResponseCommit {
        let (result_tx, result_rx) = channel();

        self.cmd_tx
            .send(AppCommand::Commit {
                result_tx,
            })
            .unwrap();
        let (height, app_hash) = result_rx.recv().unwrap().unwrap();

        abci::ResponseCommit {
            data: app_hash.to_vec(),
            // TODO: I don't really know what retain_height means. I assume it
            // means the block height that was just committed.
            retain_height: height,
        }
    }
}

/// Casting CosmWasm event attributes into ABCI event attributes
fn wasm_attrs_to_abci(wasm_attrs: Vec<WasmAttribute>) -> Vec<EventAttribute> {
    wasm_attrs
        .into_iter()
        .map(|attr| EventAttribute {
            key: attr.key.into_bytes(),
            value: attr.value.into_bytes(),
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
