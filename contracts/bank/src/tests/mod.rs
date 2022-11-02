mod instantiation;
mod namespace;

use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
    Empty, OwnedDeps,
};

use crate::{
    execute,
    msg::{Balance, UpdateNamespaceMsg},
};

const OWNER: &str = "larry";

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();

    execute::init(
        deps.as_mut(),
        OWNER.into(),
        vec![
            Balance {
                address: "jake".into(),
                coins: vec![coin(12345, "uatom"), coin(23456, "uosmo")],
            },
            Balance {
                address: "pumpkin".into(),
                coins: vec![coin(34567, "uatom"), coin(45678, "umars")],
            },
        ],
        vec![
            UpdateNamespaceMsg {
                namespace: "".into(),
                admin: Some("gov".into()),
                after_send_hook: None,
            },
            UpdateNamespaceMsg {
                namespace: "ibc".into(),
                admin: Some("ibc-transfer".into()),
                after_send_hook: None,
            },
            UpdateNamespaceMsg {
                namespace: "factory".into(),
                admin: Some("token-factory".into()),
                after_send_hook: Some("token-factory".into()),
            },
        ],
    )
    .unwrap();

    deps
}
