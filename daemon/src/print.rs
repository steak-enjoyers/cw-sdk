use cw_sdk::auth::ACCOUNT_PREFIX;

use crate::Key;

/// Print a signing key
///
/// For now we print the pubkey in hex encoding. Should we use base64 instead of be consistent with
/// Go SDK?
pub fn key(key: &Key) {
    println!("- name: {}", key.name);
    println!("  address: {}", key.address().bech32(ACCOUNT_PREFIX).unwrap());
    println!("  pubkey: {}", hex::encode(key.pubkey().to_bytes().as_slice()));
}

/// Print multiple signing keys, sorted alphabetically by name
pub fn keys(keys: &[Key]) {
    if keys.is_empty() {
        println!("[]");
    } else {
        // TODO: sort keys by name?
        keys.iter().for_each(self::key);
    }
}

/// Print a BIP-38 mnemonic phrase
pub fn mnemonic(phrase: &str) {
    let words = phrase.split(' ').collect::<Vec<_>>();
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

/// Print a serializable object as YAML
pub fn yaml(data: impl serde::Serialize) {
    let data_str = serde_yaml::to_string(&data).unwrap();
    println!("{}", data_str);
}

/// Print a serializable object as pretty JSON
pub fn json(data: impl serde::Serialize) {
    let data_str = serde_json::to_string_pretty(&data).unwrap();
    println!("{}", data_str);
}

/// Print a horizontal ruler
pub fn hr() {
    println!("--------------------");
}
