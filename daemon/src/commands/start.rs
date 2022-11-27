use std::path::Path;
use std::sync::mpsc;

use clap::Args;
use tendermint_abci::ServerBuilder;
use tracing::info;

use cw_server::{App, AppDriver};
use cw_state_machine::StateMachine;
use cw_store::Store;

use crate::{AppConfig, DaemonError};

#[derive(Args)]
pub struct StartCmd;

impl StartCmd {
    pub fn run(&self, home_dir: &Path) -> Result<(), DaemonError> {
        if !home_dir.exists() {
            return Err(DaemonError::file_not_found(home_dir)?);
        }

        // load config from disk
        let app_cfg = AppConfig::load(home_dir)?;
        info!("Loaded application config");

        // load merk store from disk
        let store = Store::open(home_dir.join("./data"))?;
        info!("Loaded Merk store");

        // create a new state machine instance wrapping the store
        let state_machine = StateMachine::new(store);

        // create a channel between the App and AppDriver
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let app = App {
            cmd_tx,
        };
        let driver = AppDriver {
            state_machine,
            cmd_rx,
        };

        // create the ABCI server
        let server = ServerBuilder::default().bind(app_cfg.listen_addr, app)?;

        // spin up the App and AppDriver
        std::thread::spawn(move || server.listen().unwrap());

        // NOTE: in basecoin, the app driver is spawned in threads.
        // here we can't do the same because the Merk store is not thread safe.
        driver.run();

        Ok(())
    }
}
