use cosmwasm_std::Addr;
use serde::Serialize;

use crate::{DaemonError, Key};

/// Print a BIP-38 mnemonic phrase
pub fn mnemonic(phrase: &str) {
    let words = phrase.split(' ').collect::<Vec<_>>();
    let word_amount = words.len();
    let mut start = 0usize;
    while start < word_amount {
        let end = (start + 4).min(word_amount);
        let slice = words[start..end]
            .iter()
            .map(|word| format!("{word: >8}"))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{: >2} - {end: >2}  {slice}", start + 1);
        start = end;
    }
}

/// Print a serializable object as pretty JSON
pub fn json(data: impl serde::Serialize) -> Result<(), DaemonError> {
    let data_str = serde_json::to_string_pretty(&data)?;
    println!("{data_str}");
    Ok(())
}

/// Print a signing key
pub fn key(key: &Key) -> Result<(), DaemonError> {
    json(PrintableKey::try_from(key)?)
}

/// Print multiple signing keys, sorted alphabetically by name
pub fn keys(keys: &[Key]) -> Result<(), DaemonError> {
    json(keys
        .iter()
        .map(PrintableKey::try_from)
        .collect::<Result<Vec<_>, _>>()?)
}

#[derive(Serialize)]
struct PrintableKey<'a> {
    pub name: &'a str,
    pub address: Addr,
    /// Hex-encoded bytearray
    pub pubkey: String,
}

impl<'a> TryFrom<&'a Key> for PrintableKey<'a> {
    type Error = DaemonError;

    fn try_from(key: &'a Key) -> Result<Self, Self::Error> {
        Ok(Self {
            name: &key.name,
            address: key.address()?,
            pubkey: hex::encode(key.pubkey().to_bytes().as_slice()),
        })
    }
}
