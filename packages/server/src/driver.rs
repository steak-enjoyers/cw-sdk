use std::sync::mpsc::Receiver;

use cw_state_machine::StateMachine;

use crate::AppCommand;

/// The driver is a wrapper around the actual state machine.
/// It maintains a channel with the ABCI server, and performs actions or queries
/// on the state machine on request for the ABCI server.
pub struct AppDriver {
    pub state_machine: StateMachine,
    pub cmd_rx: Receiver<AppCommand>,
}

impl AppDriver {
    pub fn run(&mut self) {
        loop {
            match self.cmd_rx.recv().unwrap() {
                AppCommand::Info {
                    result_tx,
                } => result_tx.send(self.state_machine.info()).unwrap(),
                AppCommand::InitChain {
                    chain_id,
                    gen_state,
                    result_tx,
                } => result_tx.send(self.state_machine.init_chain(chain_id, gen_state)).unwrap(),
                AppCommand::Query {
                    query,
                    result_tx,
                } => result_tx.send(self.state_machine.query(query)).unwrap(),
                AppCommand::BeginBlock {
                    block,
                    result_tx,
                } => result_tx.send(self.state_machine.begin_block(block)).unwrap(),
                AppCommand::DeliverTx {
                    tx,
                    result_tx,
                } => result_tx.send(self.state_machine.deliver_tx(tx)).unwrap(),
                AppCommand::Commit {
                    result_tx,
                } => result_tx.send(self.state_machine.commit()).unwrap(),
            }
        }
    }
}
