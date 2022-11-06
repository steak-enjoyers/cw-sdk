use cosmwasm_std::{testing::mock_info, to_binary, SubMsg, Uint128, WasmMsg};
use cw_bank::msg as bank;

use crate::{
    error::ContractError,
    execute,
    tests::{setup_test, BANK, DENOM},
};

const BAD_GUY: &str = "badguy";

#[test]
fn not_admin() {
    let mut deps = setup_test();

    let err = execute::mint(
        deps.as_mut(),
        mock_info(BAD_GUY, &[]),
        BAD_GUY.into(),
        DENOM.into(),
        Uint128::new(999999),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::not_token_admin(DENOM));
}

#[test]
fn minting() {
    let mut deps = setup_test();

    let res = execute::mint(
        deps.as_mut(),
        mock_info("jake", &[]),
        "someone".into(),
        DENOM.into(),
        Uint128::new(999999),
    )
    .unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: BANK.into(),
            msg: to_binary(&bank::ExecuteMsg::Mint {
                to: "someone".into(),
                denom: DENOM.into(),
                amount: Uint128::new(999999)
            })
            .unwrap(),
            funds: vec![],
        })],
    );
}

#[test]
fn burning() {
    let mut deps = setup_test();

    let res = execute::burn(
        deps.as_mut(),
        mock_info("jake", &[]),
        "someone".into(),
        DENOM.into(),
        Uint128::new(999999),
    )
    .unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: BANK.into(),
            msg: to_binary(&bank::ExecuteMsg::Burn {
                from: "someone".into(),
                denom: DENOM.into(),
                amount: Uint128::new(999999)
            })
            .unwrap(),
            funds: vec![],
        })],
    );
}

#[test]
fn force_transferring() {
    let mut deps = setup_test();

    let res = execute::force_transfer(
        deps.as_mut(),
        mock_info("jake", &[]),
        "alice".into(),
        "bob".into(),
        DENOM.into(),
        Uint128::new(999999),
    )
    .unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: BANK.into(),
            msg: to_binary(&bank::ExecuteMsg::ForceTransfer {
                from: "alice".into(),
                to: "bob".into(),
                denom: DENOM.into(),
                amount: Uint128::new(999999)
            })
            .unwrap(),
            funds: vec![],
        })],
    );
}
