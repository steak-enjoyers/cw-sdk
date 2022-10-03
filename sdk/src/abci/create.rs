use std::sync::mpsc::channel;

use tendermint_abci::{Server, ServerBuilder};

use crate::State;
use super::{App, AppDriver};

pub fn create_abci_app(state: State, listen_addr: String) -> (Server<App>, AppDriver) {
    let (cmd_tx, cmd_rx) = channel();

    let app = App {
        cmd_tx,
    };
    let driver = AppDriver {
        state,
        cmd_rx,
    };

    let server = ServerBuilder::default().bind(listen_addr, app).unwrap();

    (server, driver)
}
