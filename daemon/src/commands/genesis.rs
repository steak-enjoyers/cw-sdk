use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser, Runnable)]
pub enum GenesisCmd {
    /// Add a wasm binary code to the genesis state
    Store(StoreCmd),
    /// Instantiate a contract as part of the genesis state
    Instantiate(InstantiateCmd),
    /// Execute a contract as part of the genesis state
    Execute(ExecuteCmd),
}

#[derive(Command, Debug, Parser)]
pub struct StoreCmd {}

#[derive(Command, Debug, Parser)]
pub struct InstantiateCmd {}

#[derive(Command, Debug, Parser)]
pub struct ExecuteCmd {}

super::runnable_unimplemented!(StoreCmd);
super::runnable_unimplemented!(InstantiateCmd);
super::runnable_unimplemented!(ExecuteCmd);
