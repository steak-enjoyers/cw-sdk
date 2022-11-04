use thiserror::Error;

#[derive(Debug, Error)]
#[cfg_attr(any(test, feature = "integration-test"), derive(PartialEq))]
pub enum DenomError {
    #[error("invalid denom or namespace `{denom}`: contains empty parts")]
    EmptyParts {
        denom: String,
    },

    #[error("invalid denom or namespace `{denom}`: too long or too short")]
    IllegalLength {
        denom: String,
    },

    #[error("invalid denom or namespace `{denom}`: starts with a number")]
    LeadingNumber {
        denom: String,
    },

    #[error("invalid denom or namespace `{denom}`: contains non-alphanumeric characters")]
    NotAlphanumeric {
        denom: String,
    },
}

impl DenomError {
    pub fn empty_parts(denom: impl Into<String>) -> Self {
        Self::EmptyParts {
            denom: denom.into(),
        }
    }

    pub fn illegal_length(denom: impl Into<String>) -> Self {
        Self::IllegalLength {
            denom: denom.into(),
        }
    }

    pub fn leading_number(denom: impl Into<String>) -> Self {
        Self::LeadingNumber {
            denom: denom.into(),
        }
    }

    pub fn not_alphanumeric(denom: impl Into<String>) -> Self {
        Self::NotAlphanumeric {
            denom: denom.into(),
        }
    }
}
