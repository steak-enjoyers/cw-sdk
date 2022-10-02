use std::collections::BTreeMap;

#[derive(Debug)]
pub struct AppState {
    pub code_count: u64,
    pub codes: BTreeMap<u64, Vec<u8>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            code_count: 0,
            codes: BTreeMap::new(),
        }
    }
}
