use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser, Runnable)]
pub enum TxCmd {
    /// Upload a wasm byte code
    Store(StoreCmd),
    /// Instantiate a wasm contract
    Instantiate(InstantiateCmd),
    /// Execute a command on a wasm contract
    Execute(ExecuteCmd),
    /// Migrate a wasm contract to a new code version
    Migrate(MigrateCmd),
}

#[derive(Command, Debug, Parser)]
pub struct StoreCmd {}

#[derive(Command, Debug, Parser)]
pub struct InstantiateCmd {}

#[derive(Command, Debug, Parser)]
pub struct ExecuteCmd {}

#[derive(Command, Debug, Parser)]
pub struct MigrateCmd {}

super::runnable_unimplemented!(StoreCmd);
super::runnable_unimplemented!(InstantiateCmd);
super::runnable_unimplemented!(ExecuteCmd);
super::runnable_unimplemented!(MigrateCmd);
