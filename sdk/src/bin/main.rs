use structopt::StructOpt;
use tracing_subscriber::filter::LevelFilter;

use cw_sdk::abci::create_abci_app;
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

    let state = State::default();
    let listen_addr = format!("{}:{}", opt.host, opt.port);
    let (server, driver) = create_abci_app(state, listen_addr);

    std::thread::spawn(move || driver.run());
    server.listen().unwrap();
}
