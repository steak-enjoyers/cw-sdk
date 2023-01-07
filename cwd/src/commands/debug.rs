use clap::{Args, Subcommand};
use cw_sdk::address;

use crate::DaemonError;

#[derive(Args)]
pub struct DebugCmd {
    #[command(subcommand)]
    subcommand: DebugSubcmd,
}

#[derive(Subcommand)]
pub enum DebugSubcmd {
    /// Derive a base account's address based on its pubkey
    DeriveBaseAddress {
        /// Public key in either hex encoding
        pubkey: String,

        // TODO: add a `--base64` flag to allow using base64-encoded pubkeys
    },

    /// Derive a contract's address based on its label
    DeriveContractAddress {
        /// Contract label
        label: String,
    },
}

impl DebugCmd {
    pub fn run(self) -> Result<(), DaemonError> {
        match self.subcommand {
            DebugSubcmd::DeriveBaseAddress {
                pubkey,
            } => {
                let pubkey_bytes = hex::decode(&pubkey)?;
                let addr = address::derive_from_pubkey(&pubkey_bytes)?;
                println!("{addr}");
            },

            DebugSubcmd::DeriveContractAddress {
                label,
            } => {
                let addr = address::derive_from_label(&label)?;
                println!("{addr}");
            },
        }

        Ok(())
    }
}
