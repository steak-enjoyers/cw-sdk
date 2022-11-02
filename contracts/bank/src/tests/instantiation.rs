use cosmwasm_std::{coin, testing::mock_dependencies};

use crate::{
    denom::DenomError,
    error::ContractError,
    execute,
    msg::{Balance, Config, UpdateNamespaceMsg},
    query,
};

pub const OWNER: &str = "larry";

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies();

    let namespaces = vec![
        UpdateNamespaceMsg {
            namespace: "".into(),
            admin: Some("gov".into()),
            after_send_hook: None,
        },
        UpdateNamespaceMsg {
            namespace: "factory".into(),
            admin: Some("token-factory".into()),
            after_send_hook: Some("token-factory".into()),
        },
    ];

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
        namespaces.clone(),
    )
    .unwrap();

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
            coin(46912, "uatom"), // 12345 + 34567
            coin(45678, "umars"),
            coin(23456, "uosmo"),
        ],
    );

    let balances = query::balances(deps.as_ref(), "jake".into(), None, None).unwrap();
    assert_eq!(balances, vec![coin(12345, "uatom"), coin(23456, "uosmo")]);

    let namespace_cfgs = query::namespaces(deps.as_ref(), None, None).unwrap();
    assert_eq!(namespace_cfgs, namespaces);
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
