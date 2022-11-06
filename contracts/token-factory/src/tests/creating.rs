use cosmwasm_std::{coin, coins, testing::mock_info, Addr, Coin, DepsMut, Storage, Uint128};
use cw_bank::denom::DenomError;
use cw_utils::PaymentError;

use crate::{
    error::ContractError,
    execute,
    msg::{TokenConfig, TokenResponse},
    query,
    state::TOKEN_CONFIGS,
    tests::{fee, setup_test},
};

fn setup_token(store: &mut dyn Storage) -> (Addr, String, String) {
    let creator = Addr::unchecked("larry");
    let nonce = "uastro";

    TOKEN_CONFIGS
        .save(
            store,
            (&creator, nonce),
            &TokenConfig {
                admin: Some(Addr::unchecked("jake")),
                after_transfer_hook: Some(Addr::unchecked("pumpkin")),
            },
        )
        .unwrap();

    (creator.clone(), nonce.into(), format!("factory/{creator}/{nonce}"))
}

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

    let denom = "factory/larry/uastro";

    execute::create_token(
        deps.as_mut(),
        mock_info("larry", &[fee()]),
        "uastro".into(),
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

    let (creator, nonce, _) = setup_token(deps.as_mut().storage);

    let err = execute::create_token(
        deps.as_mut(),
        mock_info(creator.as_str(), &[fee()]),
        nonce.clone(),
        "larry".into(),
        None,
    )
    .unwrap_err();

    assert_eq!(err, ContractError::token_exists(format!("factory/larry/{nonce}")).into());
}

#[test]
fn not_owner_or_admin() {
    let mut deps = setup_test();

    let (_, _, denom) = setup_token(deps.as_mut().storage);

    let err = execute::update_token(
        deps.as_mut(),
        mock_info("badguy", &[]),
        denom.clone(),
        None,
        None,
    )
    .unwrap_err();

    assert_eq!(err, ContractError::not_token_admin(denom));
}

#[test]
fn proper_token_update() {
    let mut deps = setup_test();

    let (_, _, denom) = setup_token(deps.as_mut().storage);

    execute::update_token(
        deps.as_mut(),
        mock_info("jake", &[]),
        denom.clone(),
        None,
        Some("some_contract".into()),
    )
    .unwrap();

    let token = query::token(deps.as_ref(), denom.clone()).unwrap();
    assert_eq!(
        token,
        TokenResponse {
            denom,
            admin: None,
            after_transfer_hook: Some("some_contract".into()),
        },
    );
}
