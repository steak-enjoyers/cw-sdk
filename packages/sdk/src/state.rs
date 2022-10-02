use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct AppState {
    pub code_count: u64,
    pub codes: BTreeMap<u64, Vec<u8>>,
}
