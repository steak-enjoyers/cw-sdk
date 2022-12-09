use std::{fs, path::Path};

use clap::Args;
use tracing::info;

use crate::{path::stringify, DaemonError};

#[derive(Args)]
pub struct ResetCmd;

impl ResetCmd {
    pub fn run(&self, home_dir: &Path) -> Result<(), DaemonError> {
        let data_dir = home_dir.join("data");

        // Rust doesn't provide a function to delete all files under a folder
        // but not the folder itself.
        // Therefore we have to delete the whole folder and recreate an empty
        // one with the same name.
        fs::remove_dir_all(&data_dir)?;
        fs::create_dir(&data_dir)?;

        info!("Deleted application database at {}", stringify(&data_dir)?);

        Ok(())
    }
}
