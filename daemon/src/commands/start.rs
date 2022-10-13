use std::path::Path;
use std::sync::mpsc;

use clap::Args;
use tendermint_abci::ServerBuilder;
use tracing::error;

use cw_sdk::abci::{App, AppDriver};
use cw_sdk::state::State;

use crate::{stringify_pathbuf, AppConfig};

#[derive(Args)]
pub struct StartCmd;

impl StartCmd {
    pub fn run(&self, home_dir: &Path) {
        if !home_dir.exists() {
            error!("home directory does not exist: {}", stringify_pathbuf(home_dir));
            return;
        }

        let app_cfg = AppConfig::load(home_dir).unwrap();

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
        ServerBuilder::default().bind(app_cfg.listen_addr, app).unwrap().listen().unwrap();
    }
}
