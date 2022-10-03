use std::sync::mpsc::channel;

use structopt::StructOpt;
use tendermint_abci::ServerBuilder;
use tracing_subscriber::filter::LevelFilter;

use cw_sdk::abci::{App, AppDriver};
use cw_sdk::State;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Bind the TCP server to this host.
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Bind the TCP server to this port.
    #[structopt(short, long, default_value = "26658")]
    port: u16,

    /// Increase output logging verbosity to DEBUG level.
    #[structopt(short, long)]
    verbose: bool,

    /// Suppress all output logging (overrides --verbose).
    #[structopt(short, long)]
    quiet: bool,
}

fn main() {
    let opt: Opt = Opt::from_args();

    let log_level = if opt.quiet {
        LevelFilter::OFF
    } else if opt.verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    let (cmd_tx, cmd_rx) = channel();
    let app = App {
        cmd_tx,
    };
    let driver = AppDriver {
        state: State::default(),
        cmd_rx,
    };

    let listen_addr = format!("{}:{}", opt.host, opt.port);
    let server = ServerBuilder::default().bind(listen_addr, app).unwrap();

    std::thread::spawn(move || driver.run());
    server.listen().unwrap();
}
