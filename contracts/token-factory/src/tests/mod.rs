mod fee;
mod instantiation;
mod mocks;

use std::marker::PhantomData;

use cosmwasm_std::{
    coin,
    testing::{MockApi, MockStorage},
    Empty, OwnedDeps,
};

use crate::{execute, msg::Config};

const OWNER: &str = "larry";
const BANK: &str = "bank";

fn setup_test() -> OwnedDeps<MockStorage, MockApi, mocks::MockQuerier, Empty> {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: mocks::MockQuerier::default(),
        custom_query_type: PhantomData,
    };

    execute::init(
        deps.as_mut(),
        Config {
            owner: OWNER.into(),
            bank: BANK.into(),
            token_creation_fee: Some(coin(12345, "ujuno")),
        },
    )
    .unwrap();

    deps
}
