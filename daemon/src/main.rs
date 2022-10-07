use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::filter::LevelFilter;

use cw_daemon::{default_home, InitCmd, StartCmd, KeysCmd};

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
    /// Initialize application home directory
    Init(InitCmd),
    /// Manage private keys
    Keys(KeysCmd),
    /// Start the ABCI server
    Start(StartCmd),
}

fn main() {
    let cli = Cli::parse();

    // set home directory
    let home_dir = cli.home.unwrap_or_else(default_home);

    // set log level
    let log_level = if cli.debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    match &cli.command {
        Command::Init(cmd) => cmd.run(&home_dir),
        Command::Keys(cmd) => cmd.run(&home_dir),
        Command::Start(cmd) => cmd.run(&home_dir),
    }
}
