use cosmwasm_std::{coin, coins, Addr, Coin};
use cw_multi_test::{App, ContractWrapper, Executor};

use cw_bank::msg::{self as bank, UpdateNamespaceMsg};
use cw_token_factory::msg::NAMESPACE;

pub const OWNER: &str = "larry";

pub struct TestSuite {
    pub app: App,
    pub bank: Addr,
    pub factory: Addr,
}

impl TestSuite {
    pub fn owner() -> Addr {
        Addr::unchecked(OWNER)
    }

    pub fn setup() -> Self {
        let mut app = App::default();

        let deployer = Addr::unchecked(OWNER);

        // deploy bank contract
        let bank = {
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

        let factory = {
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
                    bank: bank.to_string(),
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
            bank.clone(),
            &bank::ExecuteMsg::UpdateNamespace(UpdateNamespaceMsg {
                namespace: NAMESPACE.into(),
                admin: Some(factory.to_string()),
                after_send_hook: Some(factory.to_string()),
            }),
            &[],
        )
        .unwrap();

        Self {
            app,
            bank,
            factory,
        }
    }

    pub fn mint_coins(&mut self, to: &Addr, coins: Vec<Coin>) {
        for coin in coins {
            self.app
                .execute_contract(
                    Self::owner(),
                    self.bank.clone(),
                    &bank::ExecuteMsg::Mint {
                        to: to.to_string(),
                        denom: coin.denom,
                        amount: coin.amount,
                    },
                    &[],
                )
                .unwrap();
        }
    }

    pub fn assert_balances(&self, user: impl Into<String>, expected: Vec<Coin>) {
        let balances: Vec<Coin> = self
            .app
            .wrap()
            .query_wasm_smart(
                self.bank.clone(),
                &bank::QueryMsg::Balances {
                    address: user.into(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(balances, expected);
    }
}
