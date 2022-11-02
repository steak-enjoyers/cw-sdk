use cosmwasm_std::{
    coin, testing::mock_info, Addr, OverflowError, OverflowOperation, StdError, Uint128, Deps
};

use crate::{
    denom::{Denom, DenomError, Namespace},
    error::ContractError,
    execute, query,
    state::BALANCES,
};

use super::setup_test;

fn assert_supply(deps: Deps, denom: &str, expected: u128) {
    let supply = query::supply(deps, denom.into()).unwrap();
    assert_eq!(supply, coin(expected, denom));
}

fn assert_balance(deps: Deps, user: &str, denom: &str, expected: u128) {
    let balance = query::balance(deps, user.into(), denom.into()).unwrap();
    assert_eq!(balance, coin(expected, denom));
}

#[test]
fn invalid_denom() {
    let mut deps = setup_test();

    let invalid_denom = "factory//uastro"; // contains an empty part

    let err = execute::mint(
        deps.as_mut(),
        mock_info("token-factory", &[]),
        "jake".into(),
        invalid_denom.into(),
        Uint128::new(12345),
    )
    .unwrap_err();

    assert_eq!(err, DenomError::empty_parts(invalid_denom).into());
}

#[test]
fn non_exist_namespace() {
    let mut deps = setup_test();

    let err = execute::mint(
        deps.as_mut(),
        mock_info("gov", &[]),
        "jake".into(),
        "market/utoken".into(),
        Uint128::new(12345),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::non_exist_namespace(Namespace::unchecked("market")));
}

#[test]
fn non_admin() {
    let mut deps = setup_test();

    // namespace admin is `Some` but is not sender
    {
        let err = execute::mint(
            deps.as_mut(),
            mock_info("pumpkin", &[]),
            "jake".into(),
            "factory/abc".into(),
            Uint128::new(12345),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::not_namespace_admin(Namespace::unchecked("factory")));
    }

    // namespace admin is `None`
    {
        let err = execute::mint(
            deps.as_mut(),
            mock_info("pumpkin", &[]),
            "jake".into(),
            "uastro".into(),
            Uint128::new(12345),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::not_namespace_admin(Namespace::unchecked("")));
    }
}

#[test]
fn zero_amount() {
    let mut deps = setup_test();

    let denom = "factory/osmo1234abcd/uastro";

    // attempt to mint zero amount
    {
        let err = execute::mint(
            deps.as_mut(),
            mock_info("token-factory", &[]),
            "jake".into(),
            denom.into(),
            Uint128::zero(),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::zero_amount(denom));
    }

    // attempt to burn zero amount
    {
        let err = execute::burn(
            deps.as_mut(),
            mock_info("token-factory", &[]),
            "jake".into(),
            denom.into(),
            Uint128::zero(),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::zero_amount(denom));
    }
}

#[test]
fn overflow() {
    let mut deps = setup_test();

    let err = execute::burn(
        deps.as_mut(),
        mock_info("token-factory", &[]),
        "jake".into(),
        "factory/osmo1234abcd/uastro".into(),
        Uint128::new(88888),
    )
    .unwrap_err();

    assert_eq!(
        err,
        StdError::overflow(OverflowError::new(OverflowOperation::Sub, 23456, 88888)).into(),
    );
}

#[test]
fn proper_minting_and_burning() {
    let mut deps = setup_test();

    // mint an existing token of which the user already has a balance
    {
        execute::mint(
            deps.as_mut(),
            mock_info("gov", &[]),
            "pumpkin".into(),
            "uatom".into(),
            Uint128::new(54321),
        )
        .unwrap();

        assert_supply(deps.as_ref(), "uatom", 101233); // 12345 + 34567 + 54321
        assert_balance(deps.as_ref(), "pumpkin", "uatom", 88888);
    }

    // mint a newly created token
    {
        let denom = "factory/juno1111ffff/utoken";

        execute::mint(
            deps.as_mut(),
            mock_info("token-factory", &[]),
            "larry".into(),
            denom.into(),
            Uint128::new(69420),
        )
        .unwrap();

        assert_supply(deps.as_ref(), denom, 69420);
        assert_balance(deps.as_ref(), "larry", denom, 69420);
    }

    // burn a token but not to zero
    {
        let denom = "ibc/12AB34CD";

        execute::burn(
            deps.as_mut(),
            mock_info("ibc-transfer", &[]),
            "pumpkin".into(),
            denom.into(),
            Uint128::new(42069),
        )
        .unwrap();

        assert_supply(deps.as_ref(), denom, 3609);
        assert_balance(deps.as_ref(), "pumpkin", denom, 3609);
    }

    // burn a token to zero
    {
        execute::burn(
            deps.as_mut(),
            mock_info("gov", &[]),
            "jake".into(),
            "uatom".into(),
            Uint128::new(12345),
        )
        .unwrap();

        assert_supply(deps.as_ref(), "uatom", 88888);
        assert_balance(deps.as_ref(), "jake", "uatom", 0);

        // not only the balance query should return zero,
        // the user's balance should have been deleted from contract store
        let opt = BALANCES
            .may_load(deps.as_ref().storage, (&Addr::unchecked("jake"), &Denom::unchecked("uatom")))
            .unwrap();
        assert!(opt.is_none());
    }
}
