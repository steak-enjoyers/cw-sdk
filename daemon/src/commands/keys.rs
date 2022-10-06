use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser, Runnable)]
pub enum KeysCmd {
    /// Add a private key, or recover one from mnemonic phrases
    Add(AddCmd),
    /// Delete the given key
    Delete(DeleteCmd),
    /// List all keys
    List(ListCmd),
    /// Retrieve key information by name or address
    Show(ShowCmd),
}

#[derive(Command, Debug, Parser)]
pub struct AddCmd {}

#[derive(Command, Debug, Parser)]
pub struct DeleteCmd {}

#[derive(Command, Debug, Parser)]
pub struct ListCmd {}

#[derive(Command, Debug, Parser)]
pub struct ShowCmd {}

super::runnable_unimplemented!(AddCmd);
super::runnable_unimplemented!(DeleteCmd);
super::runnable_unimplemented!(ListCmd);
super::runnable_unimplemented!(ShowCmd);
