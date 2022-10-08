use std::fs;
use std::path::Path;

use clap::Args;
use tracing::{error, info};

use crate::{stringify_pathbuf, AppConfig, ClientConfig};

#[derive(Args)]
pub struct InitCmd;

impl InitCmd {
    pub fn run(&self, home_dir: &Path) {
        if home_dir.exists() {
            error!("home directory already exists at {}", stringify_pathbuf(home_dir));
            return;
        }

        fs::create_dir_all(home_dir.join("config")).unwrap();
        fs::create_dir_all(home_dir.join("data")).unwrap();
        fs::create_dir_all(home_dir.join("keys")).unwrap();

        let app_cfg = AppConfig::default();
        let app_cfg_str = toml::to_string_pretty(&app_cfg).unwrap();
        fs::write(home_dir.join("config/app.toml"), app_cfg_str).unwrap();

        let client_cfg = ClientConfig::default();
        let client_cfg_str = toml::to_string_pretty(&client_cfg).unwrap();
        fs::write(home_dir.join("config/client.toml"), client_cfg_str).unwrap();

        info!("initialized home directory at {}", stringify_pathbuf(home_dir));
    }
}
