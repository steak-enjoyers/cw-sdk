use cosmwasm_std::{
    coin,
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    to_binary, SubMsg, WasmMsg,
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

    // the contract holds no coins, cannot send
    {
        let err = execute::withdraw_fee(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            None,
        )
        .unwrap_err();

        assert_eq!(err, ContractError::NoBalance);
    }

    // give the contract some coins
    let coins = vec![coin(12345, "uatom"), coin(23456, "uosmo")];
    deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins.clone());

    // owner properly withdraws coins to themself
    {
        let res = execute::withdraw_fee(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            None,
        )
        .unwrap();

        assert_eq!(
            res.messages,
            vec![SubMsg::new(WasmMsg::Execute {
                contract_addr: "bank".into(),
                msg: to_binary(&cw_bank::msg::ExecuteMsg::Send {
                    to: OWNER.into(),
                    coins: coins.clone()
                })
                .unwrap(),
                funds: vec![],
            })],
        );
    }

    // owner properly withdraws coins to another address
    {
        let res = execute::withdraw_fee(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            Some("pumpkin".into()),
        )
        .unwrap();

        assert_eq!(
            res.messages,
            vec![SubMsg::new(WasmMsg::Execute {
                contract_addr: "bank".into(),
                msg: to_binary(&cw_bank::msg::ExecuteMsg::Send {
                    to: "pumpkin".into(),
                    coins,
                })
                .unwrap(),
                funds: vec![],
            })],
        );
    }
}
