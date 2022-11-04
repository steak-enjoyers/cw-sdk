mod instantiation;

use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
    Empty, OwnedDeps,
};

use crate::{execute, msg::Config};

const OWNER: &str = "larry";

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();

    execute::init(
        deps.as_mut(),
        Config {
            owner: OWNER.into(),
            bank: "bank".into(),
            token_creation_fee: Some(coin(12345, "ujuno")),
        },
    )
    .unwrap();

    deps
}
