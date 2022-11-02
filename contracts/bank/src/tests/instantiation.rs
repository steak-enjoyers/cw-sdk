use cosmwasm_std::{coin, testing::mock_dependencies};

use crate::{
    denom::DenomError,
    error::ContractError,
    execute,
    msg::{Balance, Config, NamespaceResponse, UpdateNamespaceMsg},
    query,
};

use super::{setup_test, OWNER};

#[test]
fn proper_instantiation() {
    let deps = setup_test();

    let cfg = query::config(deps.as_ref()).unwrap();
    assert_eq!(
        cfg,
        Config {
            owner: OWNER.into(),
        },
    );

    let supplies = query::supplies(deps.as_ref(), None, None).unwrap();
    assert_eq!(
        supplies,
        vec![
            coin(23456, "factory/osmo1234abcd/uastro"),
            coin(45678, "ibc/12AB34CD"),
            coin(46912, "uatom"), // 12345 + 34567
        ],
    );

    let balances = query::balances(deps.as_ref(), "jake".into(), None, None).unwrap();
    assert_eq!(balances, vec![coin(23456, "factory/osmo1234abcd/uastro"), coin(12345, "uatom")]);

    let namespace_cfgs = query::namespaces(deps.as_ref(), None, None).unwrap();
    assert_eq!(
        namespace_cfgs,
        vec![
            NamespaceResponse {
                namespace: "".into(),
                admin: Some("gov".into()),
                after_send_hook: None,
            },
            NamespaceResponse {
                namespace: "factory".into(),
                admin: Some("token-factory".into()),
                after_send_hook: Some("token-factory".into()),
            },
            NamespaceResponse {
                namespace: "ibc".into(),
                admin: Some("ibc-transfer".into()),
                after_send_hook: None,
            },
        ],
    );
}

#[test]
fn zero_balance() {
    let mut deps = mock_dependencies();

    let err = execute::init(
        deps.as_mut(),
        OWNER.into(),
        vec![Balance {
            address: "jake".into(),
            coins: vec![coin(12345, "uatom"), coin(0, "uosmo")],
        }],
        vec![],
    )
    .unwrap_err();

    assert_eq!(err, ContractError::zero_init_balance("jake", "uosmo"));
}

#[test]
fn duplicate_balance() {
    let mut deps = mock_dependencies();

    let err = execute::init(
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
            Balance {
                address: "jake".into(),
                coins: vec![coin(88888, "uatom")],
            },
        ],
        vec![],
    )
    .unwrap_err();

    assert_eq!(err, ContractError::duplicate_balance("jake", "uatom"));
}

#[test]
fn duplicate_namespace() {
    let mut deps = mock_dependencies();

    let err = execute::init(
        deps.as_mut(),
        OWNER.into(),
        vec![],
        vec![
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
            UpdateNamespaceMsg {
                namespace: "ibc".into(),
                admin: Some("ibc-query".into()),
                after_send_hook: None,
            },
        ],
    )
    .unwrap_err();

    assert_eq!(err, ContractError::duplicate_namespace("ibc"));
}

#[test]
fn invalid_denom() {
    let mut deps = mock_dependencies();

    let err = execute::init(
        deps.as_mut(),
        OWNER.into(),
        vec![Balance {
            address: "jake".into(),
            coins: vec![coin(12345, "uatom"), coin(23456, "123abc"), coin(34567, "uosmo")],
        }],
        vec![],
    )
    .unwrap_err();

    assert_eq!(err, DenomError::leading_number("123abc").into());
}

#[test]
fn invalid_namespace() {
    let mut deps = mock_dependencies();

    let err = execute::init(
        deps.as_mut(),
        OWNER.into(),
        vec![],
        vec![UpdateNamespaceMsg {
            namespace: "123abc".into(),
            admin: None,
            after_send_hook: None,
        }],
    )
    .unwrap_err();

    assert_eq!(err, DenomError::leading_number("123abc").into());
}
