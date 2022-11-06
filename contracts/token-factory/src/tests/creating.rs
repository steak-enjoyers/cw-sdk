use cosmwasm_std::{coin, coins, testing::mock_info, Coin, DepsMut, Uint128};
use cw_bank::denom::DenomError;
use cw_utils::PaymentError;

use crate::{
    error::ContractError,
    execute,
    msg::TokenResponse,
    query,
    tests::{fee, setup_test, DENOM, OWNER},
};

#[test]
fn incorrect_fee() {
    let mut deps = setup_test();

    fn create(deps: DepsMut, coins_sent: &[Coin]) -> ContractError {
        execute::create_token(
            deps,
            mock_info("larry", coins_sent),
            "uastro".into(),
            "larry".into(),
            None,
        )
        .unwrap_err()
    }

    // no coin sent
    assert_eq!(create(deps.as_mut(), &[]), PaymentError::NoFunds {}.into());

    // multiple coins sent
    assert_eq!(
        create(deps.as_mut(), &[coin(12345, "ujuno"), coin(88888, "umars")]),
        PaymentError::MultipleDenoms {}.into(),
    );

    // incorrect fee denom
    assert_eq!(
        create(deps.as_mut(), &coins(12345, "umars")),
        PaymentError::MissingDenom("ujuno".into()).into(),
    );

    // correct fee denom but incorrect amount
    assert_eq!(
        create(deps.as_mut(), &coins(88888, "ujuno")),
        ContractError::incorrect_fee(fee(), Uint128::new(88888)),
    );
}

#[test]
fn invalid_denom() {
    let mut deps = setup_test();

    let invalid_nonce = "5&*V48%&Vc&%";

    let err = execute::create_token(
        deps.as_mut(),
        mock_info("larry", &[fee()]),
        invalid_nonce.into(),
        "larry".into(),
        None,
    )
    .unwrap_err();

    assert_eq!(err, DenomError::not_alphanumeric(format!("factory/larry/{invalid_nonce}")).into());
}

#[test]
fn proper_token_creation() {
    let mut deps = setup_test();

    let denom = "factory/larry/umars";

    execute::create_token(
        deps.as_mut(),
        mock_info("larry", &[fee()]),
        "umars".into(),
        "jake".into(),
        Some("pumpkin".into()),
    )
    .unwrap();

    let token = query::token(deps.as_ref(), denom.into()).unwrap();
    assert_eq!(
        token,
        TokenResponse {
            denom: denom.into(),
            admin: Some("jake".into()),
            after_transfer_hook: Some("pumpkin".into()),
        },
    );
}

#[test]
fn duplicate_denom() {
    let mut deps = setup_test();

    let err = execute::create_token(
        deps.as_mut(),
        mock_info("larry", &[fee()]),
        "uastro".into(),
        "larry".into(),
        None,
    )
    .unwrap_err();

    assert_eq!(err, ContractError::token_exists(DENOM).into());
}

#[test]
fn not_owner_or_admin() {
    let mut deps = setup_test();

    let err = execute::update_token(
        deps.as_mut(),
        mock_info("badguy", &[]),
        DENOM.into(),
        None,
        None,
    )
    .unwrap_err();

    assert_eq!(err, ContractError::not_token_admin(DENOM));
}

#[test]
fn proper_token_update() {
    let mut deps = setup_test();

    // admin can update token
    {
        execute::update_token(
            deps.as_mut(),
            mock_info("jake", &[]),
            DENOM.into(),
            None,
            Some("some_contract".into()),
        )
        .unwrap();

        let token = query::token(deps.as_ref(), DENOM.into()).unwrap();
        assert_eq!(
            token,
            TokenResponse {
                denom: DENOM.into(),
                admin: None,
                after_transfer_hook: Some("some_contract".into()),
            },
        );
    }

    // contract owner can update token
    {
        execute::update_token(
            deps.as_mut(),
            mock_info(OWNER, &[]),
            DENOM.into(),
            Some(OWNER.into()),
            Some("another_contract".into()),
        )
        .unwrap();

        let token = query::token(deps.as_ref(), DENOM.into()).unwrap();
        assert_eq!(
            token,
            TokenResponse {
                denom: DENOM.into(),
                admin: Some(OWNER.into()),
                after_transfer_hook: Some("another_contract".into()),
            },
        );
    }
}
