use std::path::Path;

use thiserror::Error;

use crate::path;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error(transparent)]
    Address(#[from] cw_sdk::address::AddressError),

    #[error(transparent)]
    BCrypt(#[from] bcrypt::BcryptError),

    #[error(transparent)]
    Bip32(#[from] bip32::Error),

    #[error(transparent)]
    Ecdsa(#[from] k256::ecdsa::Error),

    #[error(transparent)]
    FromHex(#[from] hex::FromHexError),

    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Jose(#[from] josekit::JoseError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Tendermint(#[from] tendermint::Error),

    #[error(transparent)]
    TendermintAbci(#[from] tendermint_abci::Error),

    #[error(transparent)]
    TendermintRpc(#[from] tendermint_rpc::Error),

    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error("failed to determine system home directory")]
    HomeDirFailed,

    #[error("failed to stringify path")]
    PathFailed,

    #[error("password is incorrect")]
    IncorrectPassword,

    #[error("file already exists: {filename}")]
    FileExists {
        filename: String,
    },

    #[error("file not found: {filename}")]
    FileNotFound {
        filename: String,
    },

    #[error("failed to cast JWT payload to key: {reason}")]
    MalformedPayload {
        reason: String,
    },

    #[error("tx sender {address} is a contract account")]
    SenderIsContract {
        address: String,
    },

    #[error("feature is not supported yet: {feature}")]
    UnsupportedFeature {
        feature: String,
    },
}

impl DaemonError {
    pub fn file_exists(filename: &Path) -> Result<Self, Self> {
        Ok(Self::FileExists {
            filename: path::stringify(filename)?,
        })
    }

    pub fn file_not_found(filename: &Path) -> Result<Self, Self> {
        Ok(Self::FileNotFound {
            filename: path::stringify(filename)?,
        })
    }

    pub fn malformed_payload(reason: impl Into<String>) -> Self {
        Self::MalformedPayload {
            reason: reason.into(),
        }
    }

    pub fn sender_is_contract(address: impl Into<String>) -> Self {
        Self::SenderIsContract {
            address: address.into(),
        }
    }

    pub fn unsupported_feature(feature: impl Into<String>) -> Self {
        Self::UnsupportedFeature {
            feature: feature.into(),
        }
    }
}
