//! The logics of this file is largely cloned from Go SDK's file keyring:
//! https://github.com/cosmos/keyring/blob/master/file.go

use std::fs;
use std::path::{Path, PathBuf};

use josekit::{jwe, jwt};
use thiserror::Error;

use crate::{stringify_pathbuf, Key, KeyError};

/// Keyring is a wrapper around a PathBuf, which represents the directory where the encrypted key
/// files are to be saved.
pub struct Keyring(PathBuf);

impl Keyring {
    /// Create a new keyring under the given directory
    pub fn new(dir: PathBuf) -> Result<Self, KeyringError> {
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(Self(dir))
    }

    /// Return the key directory
    pub fn dir(&self) -> &Path {
        &self.0
    }

    /// Return the absolute path of a key file given the key's name.
    pub fn filename(&self, name: &str) -> PathBuf {
        let file = format!("{}.key", name);
        self.dir().join(file)
    }

    /// Unlock the keyring, return the password.
    /// Firstly, check whether a password hash file already exists:
    /// - If yes, prompt the user to enter the password, and check against the hash file;
    /// - If not, then prompt the user to enter a new password, and save the hash to the file;
    pub fn unlock(&self) -> Result<String, KeyringError> {
        let password_hash_path = self.dir().join("password_hash");
        if password_hash_path.exists() {
            let password = rpassword::prompt_password(format!(
                "Enter the password to unlock keyring `{}`: ",
                stringify_pathbuf(&self.dir())
            ))?;

            let password_hash_bytes = fs::read(&password_hash_path)?;
            let password_hash = String::from_utf8(password_hash_bytes)?;
            if bcrypt::verify(&password, &password_hash)? {
                Ok(password)
            } else {
                Err(KeyringError::IncorrectPassword)
            }
        } else {
            let password = rpassword::prompt_password(format!(
                "Enter a password to encrypt the keyring `{}`: ",
                stringify_pathbuf(&self.dir())
            ))?;

            // Go SDK uses a difficult of 2
            // We use 4 here which is smallest value allowed by the bcrypt crate
            let password_hash = bcrypt::hash(&password, 4)?;
            fs::write(&password_hash_path, &password_hash)?;

            Ok(password)
        }
    }

    /// Save an arbitrary binary data in the keyring under the given name.
    pub fn set(&self, key: &Key) -> Result<(), KeyringError> {
        let filename = self.filename(&key.name);
        if filename.exists() {
            return Err(KeyringError::file_exists(&filename));
        }

        // header
        let mut header = jwe::JweHeader::new();
        header.set_token_type("JWT");
        header.set_algorithm("PBES2-HS256+A128KW");
        header.set_content_encryption("A128CBC-HS256");

        // cast key into JWT payload
        let payload = key.clone().try_into()?;

        // encrypt { header, payload } into token
        let password = self.unlock()?;
        let encrypter = jwe::PBES2_HS256_A128KW.encrypter_from_bytes(password)?;
        let token = jwt::encode_with_encrypter(&payload, &header, &encrypter)?;

        // save the token to file
        fs::write(filename, &token)?;

        Ok(())
    }

    /// Read binary data stored in the keyring with the given name
    pub fn get(&self, name: &str) -> Result<Key, KeyringError> {
        // load the file
        let token = {
            let filename = self.filename(name);
            if !filename.exists() {
                return Err(KeyringError::file_not_found(&filename));
            }
            fs::read(&filename)?
        };

        // decrypt { header, payload } from token
        let password = self.unlock()?;
        let decrypter = jwe::PBES2_HS256_A128KW.decrypter_from_bytes(password.as_bytes())?;
        let (payload, _) = jwt::decode_with_decrypter(&token, &decrypter)?;

        // recover key from payload
        payload.try_into().map_err(KeyringError::from)
    }

    /// Read binary data of all keys stored in the keyring
    pub fn list(&self) -> Result<Vec<Key>, KeyringError> {
        let password = self.unlock()?;
        let decrypter = jwe::PBES2_HS256_A128KW.decrypter_from_bytes(password.as_bytes())?;

        self
            .dir()
            .read_dir()?
            .map(|entry| {
                let entry = entry?;
                let token = fs::read(entry.path())?;
                let (payload, _) = jwt::decode_with_decrypter(&token, &decrypter)?;
                payload.try_into().map_err(KeyringError::from)
            })
            .filter(|res| res.is_ok())
            .collect()
    }

    /// Delete a key
    pub fn delete(&self, name: &str) -> Result<(), KeyringError> {
        let filename = self.filename(name);
        if filename.exists() {
            fs::remove_file(filename).map_err(KeyringError::from)
        } else {
            Err(KeyringError::file_not_found(&filename))
        }
    }
}

#[derive(Debug, Error)]
pub enum KeyringError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    BCrypt(#[from] bcrypt::BcryptError),

    #[error("{0}")]
    Jose(#[from] josekit::JoseError),

    #[error("{0}")]
    Key(#[from] KeyError),

    #[error("password is incorrect")]
    IncorrectPassword,

    #[error("file {filename} already exists")]
    FileExists {
        filename: String,
    },

    #[error("file {filename} not found")]
    FileNotFound {
        filename: String,
    },
}

impl KeyringError {
    pub fn file_exists(filename: &Path) -> Self {
        Self::FileExists {
            filename: stringify_pathbuf(filename),
        }
    }

    pub fn file_not_found(filename: &Path) -> Self {
        Self::FileNotFound {
            filename: stringify_pathbuf(filename),
        }
    }
}
