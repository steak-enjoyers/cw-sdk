use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::error;
use tracing_subscriber::filter::LevelFilter;

use cw_daemon::commands::{GenesisCmd, InitCmd, KeysCmd, QueryCmd, ResetCmd, StartCmd, TxCmd};
use cw_daemon::{path, DaemonError};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,

    /// Application home directory
    #[clap(long)]
    pub home: Option<PathBuf>,

    /// Increase output logging verbosity to DEBUG level
    #[clap(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Utilities for preparing the genesis state
    Genesis(GenesisCmd),

    /// Initialize application home directory
    Init(InitCmd),

    /// Manage private keys
    Keys(KeysCmd),

    /// Query the application state
    #[clap(alias = "q")]
    Query(QueryCmd),

    /// Start the ABCI server
    Start(StartCmd),

    /// Sign and broadcast transactions
    Tx(TxCmd),

    /// Delete the local application data
    UnsafeResetAll(ResetCmd),
}

async fn run() -> Result<(), DaemonError> {
    let cli = Cli::parse();

    // set home directory
    let home_dir = match &cli.home {
        Some(home) => home.clone(),
        None => path::default_app_home()?,
    };

    // set log level
    let log_level = if cli.debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    match &cli.command {
        Command::Genesis(cmd) => cmd.run(),
        Command::Init(cmd) => cmd.run(&home_dir),
        Command::Keys(cmd) => cmd.run(&home_dir),
        Command::Query(cmd) => cmd.run(&home_dir).await,
        Command::Start(cmd) => cmd.run(&home_dir),
        Command::Tx(cmd) => cmd.run(&home_dir).await,
        Command::UnsafeResetAll(cmd) => cmd.run(&home_dir),
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        error!("command failed with error: {}", err);
    }
}
