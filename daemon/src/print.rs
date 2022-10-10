use cw_sdk::auth::ACCOUNT_PREFIX;

use crate::Key;

pub fn print_key(key: &Key) {
    println!("- name: {}", key.name);
    println!("  address: {}", key.address().bech32(ACCOUNT_PREFIX).unwrap());
    println!("  pubkey: {}", hex::encode(key.pubkey().to_bytes().as_slice()));
}

pub fn print_keys(keys: &[Key]) {
    if keys.is_empty() {
        println!("[]");
    } else {
        // TODO: sort keys by name?
        keys.iter().for_each(print_key);
    }
}

pub fn print_mnemonic(phrase: &str) {
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

pub fn print_as_yaml(data: impl serde::Serialize) {
    let data_str = serde_yaml::to_string(&data).unwrap();
    println!("{}", data_str);
}

pub fn print_as_json(data: impl serde::Serialize) {
    let data_str = serde_json_wasm::to_string(&data).unwrap();
    println!("{}", data_str);
}
