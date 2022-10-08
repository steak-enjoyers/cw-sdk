use std::path::Path;

use bip32::{Language, Mnemonic};
use clap::{Args, Subcommand};
use rand_core::OsRng;
use text_io::read;
use tracing::error;

use cw_sdk::auth::ACCOUNT_PREFIX;

use crate::{stringify_pathbuf, Key, Keyring};

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
                    println!("\nEnter your BIP-39 mnemonic:\n");
                    let phrase: String = read!("{}\n");
                    println!("\n");
                    Mnemonic::new(phrase, Language::English).unwrap()
                } else {
                    Mnemonic::random(&mut OsRng, Language::English)
                };

                let key = Key::from_mnemonic(name, &mnemonic, *coin_type).unwrap();
                keyring.set(&key).unwrap();

                println!("");
                print_key(&key);
                println!("");

                if !recover {
                    println!("**Important** write this mnemonic phrase in a safe place!");
                    println!("It is the only way to recover your account if you ever forget your password.");
                    println!("");
                    print_mnemonic(mnemonic.phrase());
                    println!("");
                }
            },
            KeysSubcmd::Show {
                name,
            } => {
                let key = keyring.get(name).unwrap();
                println!("");
                print_key(&key);
                println!("");
            },
            KeysSubcmd::List => {
                let keys = keyring.list().unwrap();
                println!("");
                print_keys(&keys);
                println!("");
            },
            KeysSubcmd::Delete {
                name,
            } => keyring.delete(name).unwrap(),
        }
    }
}

fn print_key(key: &Key) {
    println!("- name: {}", key.name);
    println!("  address: {}", key.address().bech32(ACCOUNT_PREFIX).unwrap());
    println!("  pubkey: {}", hex::encode(key.pubkey().to_bytes().as_slice()));
}

fn print_keys(keys: &[Key]) {
    if keys.is_empty() {
        println!("[]");
        return;
    } else {
        // TODO: sort keys by name?
        keys.iter().for_each(print_key);
    }
}

fn print_mnemonic(phrase: &str) {
    let words = phrase.split(" ").collect::<Vec<_>>();
    let word_amount = words.len();
    let mut start = 0usize;
    while start < word_amount {
        let end = (start + 4).min(word_amount);
        let slice = words[start..end]
            .iter()
            .map(|word| format!("{: >8}", word))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{: >2} - {: >2}  {}", start + 1, end, slice,);
        start = end;
    }
}
