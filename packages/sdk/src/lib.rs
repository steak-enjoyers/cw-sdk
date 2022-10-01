#[derive(Clone, Debug)]
pub struct BaseApp;

impl BaseApp {
    /// Creates a new `BaseApp` instance.
    pub fn new() -> Self {
        BaseApp
    }
}

impl tendermint_abci::Application for BaseApp {}
