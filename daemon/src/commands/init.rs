use std::fs;
use std::path::Path;

use clap::Args;
use tracing::info;

use crate::{path, AppConfig, ClientConfig, DaemonError};

#[derive(Args)]
pub struct InitCmd;

impl InitCmd {
    pub fn run(&self, home_dir: &Path) -> Result<(), DaemonError> {
        if home_dir.exists() {
            return Err(DaemonError::file_exists(home_dir)?);
        }

        fs::create_dir_all(home_dir.join("config"))?;
        fs::create_dir_all(home_dir.join("data"))?;
        fs::create_dir_all(home_dir.join("keys"))?;

        let app_cfg = AppConfig::default();
        let app_cfg_str = toml::to_string_pretty(&app_cfg)?;
        fs::write(home_dir.join("config/app.toml"), app_cfg_str)?;

        let client_cfg = ClientConfig::default();
        let client_cfg_str = toml::to_string_pretty(&client_cfg)?;
        fs::write(home_dir.join("config/client.toml"), client_cfg_str)?;

        info!("initialized home directory at {}", path::stringify(home_dir)?);
        Ok(())
    }
}
