use std::path::{Path, PathBuf};

use crate::DaemonError;

/// Return the default application home directory
pub fn default_app_home() -> Result<PathBuf, DaemonError> {
    Ok(home::home_dir()
        .ok_or(DaemonError::HomeDirFailed)?
        .join(".cw"))
}

/// Return the default Tendermint home directory
pub fn default_tm_home() -> Result<PathBuf, DaemonError> {
    Ok(home::home_dir()
        .ok_or(DaemonError::HomeDirFailed)?
        .join(".tendermint"))
}

/// Convert a `&Path` to a string.
/// See: https://stackoverflow.com/questions/37388107/how-to-convert-the-pathbuf-to-string
pub fn stringify(path: &Path) -> Result<String, DaemonError> {
    path.to_path_buf()
        .into_os_string()
        .into_string()
        .map_err(|_| DaemonError::PathFailed)
}
