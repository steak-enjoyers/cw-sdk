use std::path::Path;
use std::sync::mpsc;

use clap::Args;
use tendermint_abci::ServerBuilder;

use cw_sdk::state::State;
use cw_server::{App, AppDriver};

use crate::{AppConfig, DaemonError};

#[derive(Args)]
pub struct StartCmd;

impl StartCmd {
    pub fn run(&self, home_dir: &Path) -> Result<(), DaemonError> {
        if !home_dir.exists() {
            return Err(DaemonError::file_not_found(home_dir)?);
        }

        let app_cfg = AppConfig::load(home_dir)?;

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

        let server = ServerBuilder::default().bind(app_cfg.listen_addr, app)?;

        std::thread::spawn(move || driver.run());
        server.listen().map_err(DaemonError::from)
    }
}
