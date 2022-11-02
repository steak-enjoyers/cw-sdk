mod instantiation;
mod minting;
mod namespace;
mod transfer;

use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
    Deps, Empty, OwnedDeps,
};

use crate::{
    execute,
    msg::{Balance, UpdateNamespaceMsg},
    query,
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
                coins: vec![coin(12345, "uatom"), coin(23456, "factory/osmo1234abcd/uastro")],
            },
            Balance {
                address: "pumpkin".into(),
                coins: vec![coin(34567, "uatom"), coin(45678, "ibc/12AB34CD")],
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

fn assert_supply(deps: Deps, denom: &str, expected: u128) {
    let supply = query::supply(deps, denom.into()).unwrap();
    assert_eq!(supply, coin(expected, denom));
}

fn assert_balance(deps: Deps, user: &str, denom: &str, expected: u128) {
    let balance = query::balance(deps, user.into(), denom.into()).unwrap();
    assert_eq!(balance, coin(expected, denom));
}
