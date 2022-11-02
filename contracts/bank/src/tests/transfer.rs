use cosmwasm_std::{coin, testing::mock_info, to_binary, SubMsg, Uint128, WasmMsg};

use crate::{
    denom::NamespaceAdminExecuteMsg,
    execute, query,
    tests::{assert_supply, setup_test, OWNER},
};

#[test]
fn send() {
    let mut deps = setup_test();

    // for this test, we create another coin that has an after send hook,
    // and mint some to jake
    execute::update_namespace(
        deps.as_mut(),
        mock_info(OWNER, &[]),
        "mars".into(),
        Some("martian-council".into()),
        Some("red-bank".into()),
    )
    .unwrap();
    execute::mint(
        deps.as_mut(),
        mock_info("martian-council", &[]),
        "jake".into(),
        "mars/uxmars".into(),
        Uint128::new(69420),
    )
    .unwrap();

    // two coins have after transfer hook set; the resulting submessages should be in the same order
    // as the Vec<Coin>.
    // additionally, jake completely transfers his uatom balance, which should lead to the data
    // deleted from the contract store.
    let res = execute::send(
        deps.as_mut(),
        mock_info("jake", &[]),
        "pumpkin".into(),
        vec![
            coin(42069, "mars/uxmars"),
            coin(12345, "uatom"),
            coin(22222, "factory/osmo1234abcd/uastro"),
        ],
    )
    .unwrap();

    assert_supply(deps.as_ref(), "uatom", 46912); // 12345 + 34567, unchanged by transfer
    assert_supply(deps.as_ref(), "ibc/12AB34CD", 45678);
    assert_supply(deps.as_ref(), "mars/uxmars", 69420);

    let balances = query::balances(deps.as_ref(), "jake".into(), None, None).unwrap();
    assert_eq!(
        balances,
        vec![
            coin(1234, "factory/osmo1234abcd/uastro"), // 23456 - 22222
            coin(27351, "mars/uxmars"),                // 69420 - 42069
        ],
    );

    let balances = query::balances(deps.as_ref(), "pumpkin".into(), None, None).unwrap();
    assert_eq!(
        balances,
        vec![
            coin(22222, "factory/osmo1234abcd/uastro"),
            coin(45678, "ibc/12AB34CD"),
            coin(42069, "mars/uxmars"),
            coin(46912, "uatom"),
        ],
    );

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(WasmMsg::Execute {
                contract_addr: "red-bank".into(),
                msg: to_binary(&NamespaceAdminExecuteMsg::AfterTransfer {
                    from: "jake".into(),
                    to: "pumpkin".into(),
                    denom: "mars/uxmars".into(),
                    amount: Uint128::new(42069)
                })
                .unwrap(),
                funds: vec![],
            }),
            SubMsg::new(WasmMsg::Execute {
                contract_addr: "token-factory".into(),
                msg: to_binary(&NamespaceAdminExecuteMsg::AfterTransfer {
                    from: "jake".into(),
                    to: "pumpkin".into(),
                    denom: "factory/osmo1234abcd/uastro".into(),
                    amount: Uint128::new(22222)
                })
                .unwrap(),
                funds: vec![],
            }),
        ],
    )
}
