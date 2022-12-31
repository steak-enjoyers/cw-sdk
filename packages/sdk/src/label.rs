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
//! Here we define the concept of "raw address string", which is a string that
//! is either:
//!
//! - a contract address, or
//! - a contract label
//!
//! We know it's an address if it starts with `cw1`, or a label otherwise.
//!
//! For the convenience of users and developers, the state machine accepts raw
//! address strings instead of only addresses in many instances, for example:
//!
//! - when executing a contract (a user using the CLI, or a contract emitting
//!   a submessage in the response) the contract address may be provided as a
//!   raw address string;
//! - similarly, when querying a contract (a user using the CLI, or a contract
//!   using deps.querier);
//! - when instantiating a new contract, the admin may be a raw address string
//!   in SdkMsg::Instantiate.
//!
//! In these case, the state machine is responsible for resolving the raw
//! address, returning the real underlying address as a cosmwasm_std::Addr.

use cosmwasm_std::Addr;

use crate::address::{derive_from_label, validate, AddressError, ADDRESS_PREFIX};

/// Resolve and validate a raw address string, which may either be a contract's
/// actual address or its label.
///
/// If the raw address string starts with the prefix `cw1`, we assume it is the
/// actual address; otherwise we assume it is a label and derive the address
/// from it.
pub fn resolve_raw_address(addr_raw: &str) -> Result<Addr, AddressError> {
    if addr_raw.starts_with(&format!("{ADDRESS_PREFIX}1")) {
        validate(addr_raw)
    } else {
        derive_from_label(addr_raw)
    }
}
