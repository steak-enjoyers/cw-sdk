use cosmwasm_std::testing::mock_info;

use crate::{
    denom::DenomError,
    error::ContractError,
    execute,
    msg::NamespaceResponse,
    query,
    tests::{setup_test, OWNER},
};

#[test]
fn proper_update_namespace() {
    let mut deps = setup_test();

    // owner can create new namespaces
    {
        execute::update_namespace(
            deps.as_mut(),
            mock_info(OWNER, &[]),
            "factory".into(),
            Some("token-factory".into()),
            Some("token-factory".into()),
        )
        .unwrap();

        let namespace = query::namespace(deps.as_ref(), "factory".into()).unwrap();
        assert_eq!(
            namespace,
            NamespaceResponse {
                namespace: "factory".into(),
                admin: Some("token-factory".into()),
                after_send_hook: Some("token-factory".into()),
            },
        );
    }

    // owner can update existing namespaces
    {
        execute::update_namespace(
            deps.as_mut(),
            mock_info(OWNER, &[]),
            "factory".into(),
            None,
            Some("token-factory".into()),
        )
        .unwrap();

        let namespace = query::namespace(deps.as_ref(), "factory".into()).unwrap();
        assert_eq!(
            namespace,
            NamespaceResponse {
                namespace: "factory".into(),
                admin: None,
                after_send_hook: Some("token-factory".into()),
            },
        );
    }

    // admin can update existing namespaces
    {
        execute::update_namespace(
            deps.as_mut(),
            mock_info(OWNER, &[]),
            "ibc".into(),
            Some("ibc-transfer".into()),
            Some("some-contract".into()),
        )
        .unwrap();

        let namespace = query::namespace(deps.as_ref(), "ibc".into()).unwrap();
        assert_eq!(
            namespace,
            NamespaceResponse {
                namespace: "ibc".into(),
                admin: Some("ibc-transfer".into()),
                after_send_hook: Some("some-contract".into()),
            },
        );
    }
}

#[test]
fn non_admin() {
    let mut deps = setup_test();

    // non-owner cannot create new namespaces
    {
        let err = execute::update_namespace(
            deps.as_mut(),
            mock_info("jake", &[]),
            "factory".into(),
            None,
            None,
        )
        .unwrap_err();

        assert_eq!(err, ContractError::not_namespace_admin("\"factory\""));
    }

    // non-owner and non-admin cannot update existing namespaces
    {
        let err = execute::update_namespace(
            deps.as_mut(),
            mock_info("jake", &[]),
            "ibc".into(),
            None,
            None,
        )
        .unwrap_err();

        assert_eq!(err, ContractError::not_namespace_admin("\"ibc\""));
    }
}

#[test]
fn invalid_namespace() {
    let mut deps = setup_test();

    let err = execute::update_namespace(
        deps.as_mut(),
        mock_info(OWNER, &[]),
        "abc@123".into(),
        None,
        None,
    )
    .unwrap_err();

    assert_eq!(err, DenomError::not_alphanumeric("abc@123").into());
}
