use abscissa_core::application::{self, AppCell};
use abscissa_core::config::{self, CfgCell};
use abscissa_core::{trace, Application, FrameworkError, StandardPaths};

use crate::commands::EntryPoint;
use crate::config::DaemonConfig;

pub static APP: AppCell<Daemon> = AppCell::new();

#[derive(Default)]
pub struct Daemon {
    /// Application configuration
    config: CfgCell<DaemonConfig>,
    /// Application state
    state: application::State<Self>,
}

impl Application for Daemon {
    /// Entry point command for this application
    type Cmd = EntryPoint;

    /// Application configuration
    type Cfg = DaemonConfig;

    /// Paths to resources within the application
    type Paths = StandardPaths;

    /// Accessor for application configuration
    fn config(&self) -> config::Reader<DaemonConfig> {
        self.config.read()
    }

    /// Borrow the application state immutably
    fn state(&self) -> &application::State<Self> {
        &self.state
    }

    /// Register all components by this application.
    ///
    /// If you would like to add additional components to your application beyond the default ones
    /// provided by the framework, this is the place to do so.
    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let framework_components = self.framework_components(command)?;
        let mut app_components = self.state.components_mut();
        app_components.register(framework_components)
    }

    /// Post-configuration lifecycle callback.
    ///
    /// Called regardless of whether config is loaded to indicate this is the time in app lifecycle
    /// when configuration would be loaded if possible.
    fn after_config(&mut self, config: Self::Cfg) -> Result<(), FrameworkError> {
        let mut components = self.state.components_mut();
        components.after_config(&config)?;
        self.config.set_once(config);
        Ok(())
    }

    /// Get tracing configuration from command-line options
    fn tracing_config(&self, command: &EntryPoint) -> trace::Config {
        if command.verbose {
            trace::Config::verbose()
        } else {
            trace::Config::default()
        }
    }
}
