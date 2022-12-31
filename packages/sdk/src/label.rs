//! In cw-sdk, contract addresses are derived deterministically from the
//! contract labels.
//!
//! The state machine must be programmed to ensure these labels:
//!
//! - are unique: no two contract has the same label, or have labels that derive
//!   the same address (i.e. hash clash);
//! - do not start with the prefix `cw1`: so that they can not be confused with
//!   addresses.
//!
//! When a user or a contract provides the state machine a "raw address string",
//! e.g. when executing a contract from the CLI, or emitting a wasm execute
//! message in the response of a contract call, this raw address string may
//! either be the actual address, or the label. The state machine is responsible
//! to resolve it and returns a verified address (as `cosmwasm_std::Addr`).

use cosmwasm_std::Addr;

use crate::address::{derive_from_label, validate, AddressError, ADDRESS_PREFIX};

/// Resolve and validate a raw address string, which may either be a contract's
/// actual address or its label.
///
/// If the raw address string starts with the prefix `cw1`, we assume it is the
/// actual address; otherwise we assume it is a label and derive the address
/// from it.
pub fn resolve_raw_address(addr_raw: String) -> Result<Addr, AddressError> {
    if addr_raw.starts_with(&format!("{ADDRESS_PREFIX}1")) {
        validate(&addr_raw)
    } else {
        derive_from_label(&addr_raw)
    }
}
