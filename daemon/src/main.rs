use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::filter::LevelFilter;

use cw_daemon::{default_home, InitCmd, StartCmd};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Application home directory
    #[arg(long)]
    pub home: Option<PathBuf>,

    /// Increase output logging verbosity to DEBUG level
    #[arg(long, default_value = "false")]
    pub verbose: bool,

    /// Suppress all output logging (overrides --verbose)
    #[arg(long, default_value = "false")]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize application home directory
    Init(InitCmd),
    /// Start the ABCI server
    Start(StartCmd),
}

fn main() {
    let cli = Cli::parse();

    let home_dir = cli.home.unwrap_or_else(default_home);

    let log_level = if cli.quiet {
        LevelFilter::OFF
    } else if cli.verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    match &cli.command {
        Command::Init(cmd) => cmd.run(&home_dir),
        Command::Start(cmd) => cmd.run(&home_dir),
    }
}
