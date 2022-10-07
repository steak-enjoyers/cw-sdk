use std::path::{Path, PathBuf};

/// Default app home directory under the system home directory.
///
/// TODO: how can we make this configurable? perhaps read an env var at compilation time?
/// similar problem for bech32 account prefixes.
pub const DEFAULT_HOME: &str = ".cw";

/// Return the absolute path of the default application home directory.
/// Panic if fails to get the system home directory.
pub fn default_home() -> PathBuf {
    let sys_home = home::home_dir().expect("failed to get the system home directory");
    sys_home.join(DEFAULT_HOME)
}

/// Converts a PathBuf to a string. Panic on failure.
///
/// See: https://stackoverflow.com/questions/37388107/how-to-convert-the-pathbuf-to-string
pub fn stringify_pathbuf(path: &Path) -> String {
    path.to_path_buf().into_os_string().into_string().unwrap()
}
