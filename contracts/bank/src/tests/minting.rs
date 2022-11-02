use cosmwasm_std::{
    testing::{mock_dependencies, mock_info, MockApi, MockQuerier, MockStorage},
    Empty, OwnedDeps,
};

use crate::{
    denom::DenomError,
    error::ContractError,
    execute,
    msg::{NamespaceResponse, UpdateNamespaceMsg},
    query,
};

use super::{setup_test, OWNER};
