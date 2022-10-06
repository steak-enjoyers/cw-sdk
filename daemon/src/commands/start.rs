use std::sync::mpsc;

use abscissa_core::{Command, Runnable, Application};
use clap::Parser;
use tendermint_abci::ServerBuilder;

use cw_sdk::abci::{App, AppDriver};
use cw_sdk::State;

use crate::app::APP;

#[derive(Command, Debug, Parser)]
pub struct StartCmd {}

impl Runnable for StartCmd {
    fn run(&self) {
        let cfg = APP.config();
        let listen_addr = format!("{}:{}", cfg.host, cfg.port);

        // TODO: currently we use an in-memory mock storage, and always start the default blank
        // state when starting the daemon.
        let state = State::default();

        let (cmd_tx, cmd_rx) = mpsc::channel();
        let app = App {
            cmd_tx,
        };
        let driver = AppDriver {
            state,
            cmd_rx,
        };

        std::thread::spawn(move || driver.run());
        ServerBuilder::default().bind(listen_addr, app).unwrap().listen().unwrap();
    }
}
