use std::collections::HashMap;

use cosmwasm_std::{coin, coins, Addr, Coin};
use cw_multi_test::{App, ContractWrapper, Executor};

use cw_bank::msg::{self as bank, UpdateNamespaceMsg};
use cw_token_factory::{
    msg::{ExecuteMsg, NAMESPACE},
    error::ContractError,
};

const OWNER: &str = "larry";

#[derive(Clone, PartialEq, Eq, Hash)]
enum Contract {
    Bank,
    TokenFactory,
}

fn setup_test() -> (App, HashMap<Contract, Addr>) {
    let mut app = App::default();

    let deployer = Addr::unchecked(OWNER);

    // deploy bank contract
    let bank_addr = {
        use cw_bank::{
            contract::{execute, instantiate, query},
            msg::{Balance, InstantiateMsg},
        };

        let contract = Box::new(ContractWrapper::new(execute, instantiate, query));
        let code_id = app.store_code(contract);

        app.instantiate_contract(
            code_id,
            deployer.clone(),
            &InstantiateMsg {
                owner: OWNER.into(),
                balances: vec![Balance {
                    address: "jake".into(),
                    coins: coins(100_000, "ujuno"),
                }],
                namespace_cfgs: vec![
                    UpdateNamespaceMsg {
                        namespace: "".into(),
                        admin: Some(OWNER.into()),
                        after_send_hook: None,
                    },
                    UpdateNamespaceMsg {
                        namespace: "ibc".into(),
                        admin: Some(OWNER.into()),
                        after_send_hook: None,
                    },
                ],
            },
            &[],
            "bank",
            None,
        )
        .unwrap()
    };

    let token_factory_addr = {
        use cw_token_factory::{
            contract::{execute, instantiate, query},
            msg::InstantiateMsg,
        };

        let contract = Box::new(ContractWrapper::new(execute, instantiate, query));
        let code_id = app.store_code(contract);

        app.instantiate_contract(
            code_id,
            deployer.clone(),
            &InstantiateMsg {
                owner: OWNER.into(),
                bank: bank_addr.to_string(),
                token_creation_fee: Some(coin(12345, "ujuno")),
            },
            &[],
            "token-factory",
            None,
        )
        .unwrap()
    };

    app.execute_contract(
        deployer,
        bank_addr.clone(),
        &bank::ExecuteMsg::UpdateNamespace(UpdateNamespaceMsg {
            namespace: NAMESPACE.into(),
            admin: Some(token_factory_addr.to_string()),
            after_send_hook: Some(token_factory_addr.to_string()),
        }),
        &[],
    )
    .unwrap();

    let addresses = vec![
        (Contract::Bank, bank_addr),
        (Contract::TokenFactory, token_factory_addr),
    ]
    .into_iter()
    .collect();

    (app, addresses)
}

fn mint_coins(app: &mut App, sender: &Addr, bank: &Addr, to: &Addr, coins: Vec<Coin>) {
    for coin in coins {
        app.execute_contract(
            sender.clone(),
            bank.clone(),
            &bank::ExecuteMsg::Mint {
                to: to.into(),
                denom: coin.denom,
                amount: coin.amount,
            },
            &[],
        )
        .unwrap();
    }
}

fn assert_balances(app: &App, bank: &Addr, user: impl Into<String>, expected: Vec<Coin>) {
    let balances: Vec<Coin> = app
        .wrap()
        .query_wasm_smart(
            bank,
            &bank::QueryMsg::Balances {
                address: user.into(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(balances, expected);
}

#[test]
fn not_owner() {
    let (mut app, addresses) = setup_test();

    let non_owner = Addr::unchecked("jake");

    let err = app.execute_contract(
        non_owner,
        addresses[&Contract::TokenFactory].clone(),
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
    let (mut app, addresses) = setup_test();

    let err = app.execute_contract(
        Addr::unchecked(OWNER),
        addresses[&Contract::TokenFactory].clone(),
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
    let (mut app, addresses) = setup_test();

    let owner = Addr::unchecked(OWNER);

    mint_coins(
        &mut app,
        &owner,
        &addresses[&Contract::Bank],
        &addresses[&Contract::TokenFactory],
        vec![coin(23456, "umars"), coin(12345, "uatom")],
    );

    app.execute_contract(
        owner.clone(),
        addresses[&Contract::TokenFactory].clone(),
        &ExecuteMsg::WithdrawFee {
            to: None,
        },
        &[],
    )
    .unwrap();

    // the owner should have received funds
    assert_balances(
        &app,
        &addresses[&Contract::Bank],
        &owner,
        vec![coin(12345, "uatom"), coin(23456, "umars")],
    );
}

#[test]
fn proper_withdraw_to_else() {
    let (mut app, addresses) = setup_test();

    let owner = Addr::unchecked(OWNER);

    mint_coins(
        &mut app,
        &owner,
        &addresses[&Contract::Bank],
        &addresses[&Contract::TokenFactory],
        vec![coin(23456, "uosmo"), coin(12345, "ibc/1234ABCD")],
    );

    app.execute_contract(
        owner,
        addresses[&Contract::TokenFactory].clone(),
        &ExecuteMsg::WithdrawFee {
            to: Some("pumpkin".into()),
        },
        &[],
    )
    .unwrap();

    // pumpkin should have received funds
    assert_balances(
        &app,
        &addresses[&Contract::Bank],
        "pumpkin",
        vec![coin(12345, "ibc/1234ABCD"), coin(23456, "uosmo")],
    );
}
