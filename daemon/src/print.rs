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

/// Print a signing key
pub fn key(key: &Key) -> Result<(), DaemonError> {
    println!("- name: {}", key.name);
    println!("  address: {}", key.address()?);
    println!("  pubkey: {}", hex::encode(key.pubkey().to_bytes().as_slice()));
    Ok(())
}

/// Print multiple signing keys, sorted alphabetically by name
pub fn keys(keys: &[Key]) -> Result<(), DaemonError> {
    if keys.is_empty() {
        println!("[]");
        Ok(())
    } else {
        // TODO: sort keys by name?
        keys.iter().try_for_each(self::key)
    }
}

/// Print a serializable object as pretty JSON
pub fn json(data: impl serde::Serialize) -> Result<(), DaemonError> {
    let data_str = serde_json::to_string_pretty(&data)?;
    println!("{data_str}");
    Ok(())
}
