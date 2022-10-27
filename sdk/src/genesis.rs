use cosmwasm_schema::cw_serde;

use crate::SdkMsg;

/// This should be included inside `~/.tendermint/genesis.json`, under the `app_state` field.
/// Tendermint will provide this as JSON bytes by in the InitChain request.
/// The state machine should deserialize the bytes upon receipt.
#[derive(Default)]
#[cw_serde]
pub struct GenesisState {
    /// Address of the account which will act as the sender of genesis messages.
    ///
    /// For example, if an "instantiate" message in included in `msgs`, then the deployer address
    /// will be provided as `info.sender` in the instantiation call.
    ///
    /// Note that during genesis, no transaction authentication is performed. The application
    /// developers must provide a trusted deployer account.
    pub deployer: String,

    /// Messages to be executed in order during the InitChain call.
    pub msgs: Vec<SdkMsg>,
}
