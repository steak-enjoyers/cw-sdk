pub mod genesis;
pub mod init;
pub mod keys;
pub mod query;
pub mod start;
pub mod tx;

use std::path::PathBuf;

use abscissa_core::{Command, Configurable, FrameworkError, Runnable};
use clap::Parser;

use crate::{DaemonConfig, DEFAULT_CONFIG, DEFAULT_HOME};

#[derive(Command, Debug, Parser, Runnable)]
pub enum DaemonCommand {
    /// Utilities for preparing the genesis state
    #[clap(subcommand)]
    Genesis(genesis::GenesisCmd),

    /// Initialize app config and state
    Init(init::InitCmd),

    /// Manage your application's keys
    #[clap(subcommand)]
    Keys(keys::KeysCmd),

    /// Query subcommands
    #[clap(subcommand)]
    Query(query::QueryCmd),

    /// Run the ABCI server
    Start(start::StartCmd),

    /// Transaction subcommands
    #[clap(subcommand)]
    Tx(tx::TxCmd),
}

#[derive(Command, Debug, Parser)]
pub struct EntryPoint {
    #[clap(subcommand)]
    cmd: DaemonCommand,

    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,

    /// Use the specified config file
    #[clap(short, long)]
    pub config: Option<String>,
}

impl Runnable for EntryPoint {
    fn run(&self) {
        self.cmd.run()
    }
}

/// This trait allows you to define how application configuration is loaded.
impl Configurable<DaemonConfig> for EntryPoint {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        let filename = self
            .config
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| [DEFAULT_HOME, DEFAULT_CONFIG].iter().collect());

        // If you'd like for a missing config file to be a hard error,
        // always return `Some(filename)` here
        if filename.exists() {
            Some(filename)
        } else {
            None
        }
    }

    /// Apply changes to the config after it's be loaded, e.g. overriding values in a config file
    /// using command-line options.
    fn process_config(&self, config: DaemonConfig) -> Result<DaemonConfig, FrameworkError> {
        // for now we don't override any config params
        Ok(config)
    }
}

/// Implements the `Runnable` trait for a command, that when run, simply panics with an
/// "umimplemented" message. Used for development purposes only. Eventually this should be removed.
macro_rules! runnable_unimplemented {
    ($cmd: ty) => {
        impl Runnable for $cmd {
            fn run(&self) {
                panic!("this command is not implemented yet!");
            }
        }
    };
}

pub(crate) use runnable_unimplemented;
