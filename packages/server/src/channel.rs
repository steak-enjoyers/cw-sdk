use std::sync::mpsc::Sender;

use cosmwasm_std::{Binary, BlockInfo, Event};

use cw_sdk::{hash::HASH_LENGTH, GenesisState, SdkQuery, Tx};
use cw_state_machine::error::Result as StateMachineResult;

/// The ABCI server and the driver maintains a channel between them, and
/// communicate by sending commands.
/// This enum defines the commands allowed to be transmitted through the channel.
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Returns the last committed block height and app hash
    Info {
        result_tx: Sender<StateMachineResult<(i64, [u8; HASH_LENGTH])>>,
    },

    /// Provide the genesis state, returns the app hash.
    InitChain {
        chain_id: String,
        gen_state: GenesisState,
        result_tx: Sender<StateMachineResult<[u8; HASH_LENGTH]>>,
    },

    /// Provide the query message, returns the query response in binary format.
    Query {
        query: SdkQuery,
        result_tx: Sender<StateMachineResult<Binary>>,
    },

    /// Provide chain id, block height and time, return events emitted during
    /// the begin block process.
    BeginBlock {
        block: BlockInfo,
        result_tx: Sender<StateMachineResult<Vec<Event>>>,
    },

    /// Provide a tx, returns the events emitted during tx execution.
    DeliverTx {
        tx: Tx,
        result_tx: Sender<StateMachineResult<Vec<Event>>>,
    },

    /// Returns the block height and app hash that was committed.
    Commit {
        result_tx: Sender<StateMachineResult<(i64, [u8; HASH_LENGTH])>>,
    },
}
