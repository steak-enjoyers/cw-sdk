use std::fs;
use std::path::Path;

use clap::{Args, Parser, Subcommand};
use tracing::{error, info};

use crate::stringify_pathbuf;

#[derive(Args)]
pub struct KeysCmd {
    #[clap(subcommand)]
    pub subcommand: KeysSubcmd,
}

#[derive(Subcommand)]
pub enum KeysSubcmd {
    /// Add or recover a private key and save it to an encrypted file
    Add {
        /// A human-readable name of the key
        name: String,

        /// Provide seed phrase to recover an existing key instead of creating
        #[clap(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
        recover: bool,

        /// BIP-44 coin type for HD derivation
        #[clap(long, default_value_t = 118)]
        coin_type: u32,
    },
    /// Delete a given key
    Delete {
        /// Name of the key to delete
        name: String,
    },
    /// Display details of a key
    Show {
        /// Name of the key to show
        name: String,
    },
    /// List all keys
    List,
}

impl KeysCmd {
    pub fn run(&self, home_dir: &Path) {
        if !home_dir.exists() {
            error!("home directory does not exist: {}", stringify_pathbuf(home_dir));
            return;
        }

        let keys_dir = home_dir.join("keys");

        match &self.subcommand {
            KeysSubcmd::Add {
                name,
                recover,
                coin_type,
            } => add_key(&keys_dir, name, *recover, *coin_type),
            KeysSubcmd::Delete {
                name,
            } => delete_key(&keys_dir, name),
            KeysSubcmd::Show {
                name,
            } => show_key(&keys_dir, name),
            KeysSubcmd::List => list_keys(&keys_dir),
        }
    }
}

pub fn add_key(keys_dir: &Path, name: &str, recover: bool, coin_type: u32) {
    // generate a seed phrase, or prompt the user to input one
}

pub fn delete_key(keys_dir: &Path, name: &str) {}

pub fn show_key(keys_dir: &Path, name: &str) {}

pub fn list_keys(keys_dir: &Path) {}
