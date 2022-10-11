use std::path::Path;

use bip32::{Language, Mnemonic};
use clap::{Args, Subcommand};
use rand_core::OsRng;
use tracing::error;

use crate::{print, prompt, stringify_pathbuf, Key, Keyring};

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

        let keyring = Keyring::new(home_dir.join("keys")).unwrap();

        match &self.subcommand {
            KeysSubcmd::Add {
                name,
                recover,
                coin_type,
            } => {
                let mnemonic = if *recover {
                    let phrase: String = prompt::input("enter your BIP-39 mnemonic").unwrap();
                    println!("\n");
                    Mnemonic::new(phrase, Language::English).unwrap()
                } else {
                    Mnemonic::random(&mut OsRng, Language::English)
                };

                let key = Key::from_mnemonic(name, &mnemonic, *coin_type).unwrap();
                keyring.set(&key).unwrap();

                println!();
                print::key(&key);

                if !recover {
                    println!("**Important** write this mnemonic phrase in a safe place!");
                    println!("It is the only way to recover your account if you ever forget your password.");
                    println!();
                    print::mnemonic(mnemonic.phrase());
                }
            },
            KeysSubcmd::Show {
                name,
            } => {
                let key = keyring.get(name).unwrap();
                println!();
                print::key(&key);
            },
            KeysSubcmd::List => {
                let keys = keyring.list().unwrap();
                println!();
                print::keys(&keys);
            },
            KeysSubcmd::Delete {
                name,
            } => keyring.delete(name).unwrap(),
        }
    }
}
