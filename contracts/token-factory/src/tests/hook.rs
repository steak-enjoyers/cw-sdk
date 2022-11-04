use cosmwasm_std::{testing::mock_info, to_binary, Addr, Storage, SubMsg, Uint128, WasmMsg};

use cw_bank::msg as bank;

use crate::{
    error::ContractError,
    execute,
    msg::TokenConfig,
    state::TOKEN_CONFIGS,
    tests::{setup_test, BANK},
};

const DENOM: &str = "factory/osmo1234abcd/uastro";

fn set_hook(store: &mut dyn Storage, after_transfer_hook: Option<&str>) {
    TOKEN_CONFIGS
        .save(
            store,
            (&Addr::unchecked("osmo1234abcd"), "uastro"),
            &TokenConfig {
                admin: None,
                after_transfer_hook: after_transfer_hook.map(Addr::unchecked),
            },
        )
        .unwrap();
}

#[test]
fn not_bank() {
    let mut deps = setup_test();

    let err = execute::after_transfer(
        deps.as_mut(),
        mock_info("jake", &[]),
        "alice".into(),
        "bob".into(),
        DENOM.into(),
        Uint128::new(12345),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::NotBank);
}

#[test]
fn hook_undefined() {
    let mut deps = setup_test();

    set_hook(deps.as_mut().storage, None);

    let res = execute::after_transfer(
        deps.as_mut(),
        mock_info(BANK, &[]),
        "alice".into(),
        "bob".into(),
        DENOM.into(),
        Uint128::new(12345),
    )
    .unwrap();

    assert_eq!(res.messages, vec![]);
}

#[test]
fn hook_defined() {
    let mut deps = setup_test();

    set_hook(deps.as_mut().storage, Some("jake"));

    let res = execute::after_transfer(
        deps.as_mut(),
        mock_info(BANK, &[]),
        "alice".into(),
        "bob".into(),
        DENOM.into(),
        Uint128::new(12345),
    )
    .unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: "jake".into(),
            msg: to_binary(&bank::HookMsg::AfterTransfer {
                from: "alice".into(),
                to: "bob".into(),
                denom: DENOM.into(),
                amount: Uint128::new(12345)
            })
            .unwrap(),
            funds: vec![]
        })],
    );
}
