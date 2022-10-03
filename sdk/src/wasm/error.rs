use thiserror::Error;

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("contract uses a feature that is not yet implemented: {feature}")]
    Unimplemented {
        feature: String,
    },
}

impl WasmError {
    pub fn unimplemented(feature: impl ToString) -> Self {
        Self::Unimplemented {
            feature: feature.to_string(),
        }
    }
}
