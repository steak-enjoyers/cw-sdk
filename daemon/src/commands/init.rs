use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser)]
pub struct InitCmd {}

super::runnable_unimplemented!(InitCmd);