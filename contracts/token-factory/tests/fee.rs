mod suite;

use cosmwasm_std::{coin, Addr};
use cw_multi_test::Executor;
use cw_token_factory::{error::ContractError, msg::ExecuteMsg};

use suite::TestSuite;

#[test]
fn not_owner() {
    let mut suite = TestSuite::setup();

    let non_owner = Addr::unchecked("jake");

    let err = suite
        .app
        .execute_contract(
            non_owner,
            suite.factory.clone(),
            &ExecuteMsg::WithdrawFee {
                to: None,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(err.downcast::<ContractError>().unwrap(), ContractError::NotOwner);
}

#[test]
fn no_balance() {
    let mut suite = TestSuite::setup();

    let err = suite
        .app
        .execute_contract(
            TestSuite::owner(),
            suite.factory.clone(),
            &ExecuteMsg::WithdrawFee {
                to: None,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(err.downcast::<ContractError>().unwrap(), ContractError::NoBalance);
}

#[test]
fn proper_withdrawal_to_self() {
    let mut suite = TestSuite::setup();

    suite.mint_coins(&suite.factory.clone(), vec![coin(23456, "umars"), coin(12345, "uatom")]);

    suite
        .app
        .execute_contract(
            TestSuite::owner(),
            suite.factory.clone(),
            &ExecuteMsg::WithdrawFee {
                to: None,
            },
            &[],
        )
        .unwrap();

    // the owner should have received funds
    suite.assert_balances(&TestSuite::owner(), vec![coin(12345, "uatom"), coin(23456, "umars")]);
}

#[test]
fn proper_withdraw_to_else() {
    let mut suite = TestSuite::setup();

    suite.mint_coins(
        &suite.factory.clone(),
        vec![coin(23456, "uosmo"), coin(12345, "ibc/1234ABCD")],
    );

    suite
        .app
        .execute_contract(
            TestSuite::owner(),
            suite.factory.clone(),
            &ExecuteMsg::WithdrawFee {
                to: Some("pumpkin".into()),
            },
            &[],
        )
        .unwrap();

    // pumpkin should have received funds
    suite.assert_balances("pumpkin", vec![coin(12345, "ibc/1234ABCD"), coin(23456, "uosmo")]);
}
