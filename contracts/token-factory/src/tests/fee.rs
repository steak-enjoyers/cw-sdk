use cosmwasm_std::{
    coin,
    testing::{mock_env, mock_info},
};

use crate::{
    error::ContractError,
    execute, query,
    tests::{setup_test, OWNER},
};

#[test]
fn updating_fee() {
    let mut deps = setup_test();

    // non-owner cannot update fee
    {
        let err = execute::update_fee(deps.as_mut(), mock_info("jake", &[]), None).unwrap_err();
        assert_eq!(err, ContractError::NotOwner);
    }

    // owner properly updates fee
    {
        let fee = Some(coin(88888, "umars"));

        execute::update_fee(deps.as_mut(), mock_info(OWNER, &[]), fee.clone()).unwrap();

        let cfg = query::config(deps.as_ref()).unwrap();
        assert_eq!(cfg.token_creation_fee, fee);
    }
}

#[test]
fn withdrawing_fee() {
    let mut deps = setup_test();

    // non-owner cannot withdraw fees
    {
        let err = execute::withdraw_fee(
            deps.as_mut(),
            mock_env(),
            mock_info("jake", &[]),
            None,
        )
        .unwrap_err();

        assert_eq!(err, ContractError::NotOwner);
    }

    // further tests require querying the bank contract
    // for those we move to integration tests instead
}
