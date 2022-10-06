mod app;
mod commands;
mod config;

pub use app::*;
pub use commands::*;
pub use config::*;

/// Default app home directory
pub const DEFAULT_HOME: &str = "~/.cw";

/// Default config filename under the home directory
pub const DEFAULT_CONFIG: &str = "config/config.toml";
