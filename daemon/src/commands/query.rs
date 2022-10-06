use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser, Runnable)]
pub enum QueryCmd {
    /// Query a single account
    Account(AccountCmd),
    /// Query all accounts
    Accounts(AccountsCmd),
    /// Query metadata of a single wasm byte code
    Code(CodeCmd),
    /// Query metadata of all wasm byte codes
    Codes(CodesCmd),
    /// Query metadata of a single contract
    Contract(ContractCmd),
    /// Query metadata of all contracts
    Contracts(ContractsCmd),
    /// Perform a raw query on a contract
    Raw(WasmRawCmd),
    /// Perform a smart query on a contract
    Smart(WasmSmartCmd),
}

#[derive(Command, Debug, Parser)]
pub struct AccountCmd {}

#[derive(Command, Debug, Parser)]
pub struct AccountsCmd {}

#[derive(Command, Debug, Parser)]
pub struct CodeCmd {}

#[derive(Command, Debug, Parser)]
pub struct CodesCmd {}

#[derive(Command, Debug, Parser)]
pub struct ContractCmd {}

#[derive(Command, Debug, Parser)]
pub struct ContractsCmd {}

#[derive(Command, Debug, Parser)]
pub struct WasmRawCmd {}

#[derive(Command, Debug, Parser)]
pub struct WasmSmartCmd {}

super::runnable_unimplemented!(AccountCmd);
super::runnable_unimplemented!(AccountsCmd);
super::runnable_unimplemented!(CodeCmd);
super::runnable_unimplemented!(CodesCmd);
super::runnable_unimplemented!(ContractCmd);
super::runnable_unimplemented!(ContractsCmd);
super::runnable_unimplemented!(WasmRawCmd);
super::runnable_unimplemented!(WasmSmartCmd);
