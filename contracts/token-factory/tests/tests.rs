use cosmwasm_std::{coin, coins, Addr, Coin, Empty, Event, Uint128};
use cw_bank::msg::{self as bank, UpdateNamespaceMsg};
use cw_multi_test::{App, ContractWrapper, Executor};
use cw_token_factory::{
    error::ContractError,
    msg::{ExecuteMsg, NAMESPACE},
};

const OWNER: &str = "larry";
const DENOM: &str = "factory/jake/umars";

mod mock_hook_handler {
    use cosmwasm_std::{
        Binary, Deps, DepsMut, Empty, Env, Event, MessageInfo, Response, StdResult,
    };
    use cw_bank::msg::HookMsg;

    pub fn instantiate(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }

    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: HookMsg,
    ) -> StdResult<Response> {
        match msg {
            HookMsg::AfterTransfer {
                from,
                to,
                denom,
                amount,
            } => {
                let event = Event::new("mock_hook_handle")
                    .add_attribute("from", from.to_string())
                    .add_attribute("to", to.to_string())
                    .add_attribute("coin", format!("{amount}{}", denom.to_string()));
                Ok(Response::new().add_event(event))
            },
        }
    }

    pub fn query(_deps: Deps, _env: Env, _query: Empty) -> StdResult<Binary> {
        panic!("[mock]: unimplemented");
    }
}

struct TestSuite {
    pub app: App,
    pub bank: Addr,
    pub factory: Addr,
    pub hook_handler: Addr,
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
                contract::{execute, instantiate, query, sudo},
                msg::{Balance, InstantiateMsg},
            };

            let contract = Box::new(ContractWrapper::new(execute, instantiate, query).with_sudo(sudo));
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
                            after_transfer_hook: None,
                        },
                        UpdateNamespaceMsg {
                            namespace: "ibc".into(),
                            admin: Some(OWNER.into()),
                            after_transfer_hook: None,
                        },
                    ],
                },
                &[],
                "bank",
                None,
            )
            .unwrap()
        };

        // deploy token factory contract
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
                    token_creation_fee: None,
                },
                &[],
                "token-factory",
                None,
            )
            .unwrap()
        };

        // deploy a mock hook handler contract
        let hook_handler = {
            use mock_hook_handler::{execute, instantiate, query};

            let contract = Box::new(ContractWrapper::new(execute, instantiate, query));
            let code_id = app.store_code(contract);

            app.instantiate_contract(
                code_id,
                deployer.clone(),
                &Empty {},
                &[],
                "hook-handler",
                None,
            )
            .unwrap()
        };

        // set token factory contract as the admin of "factory" namespace
        app.execute_contract(
            deployer,
            bank.clone(),
            &bank::ExecuteMsg::UpdateNamespace(UpdateNamespaceMsg {
                namespace: NAMESPACE.into(),
                admin: Some(factory.to_string()),
                after_transfer_hook: Some(factory.to_string()),
            }),
            &[],
        )
        .unwrap();

        // create a token
        app.execute_contract(
            Addr::unchecked("jake"),
            factory.clone(),
            &ExecuteMsg::CreateToken {
                nonce: "umars".into(),
                admin: "pumpkin".into(),
                after_transfer_hook: Some(hook_handler.to_string()),
            },
            &[],
        )
        .unwrap();

        // mint some mars tokens to alice
        app.execute_contract(
            Addr::unchecked("pumpkin"),
            factory.clone(),
            &ExecuteMsg::Mint {
                to: "alice".into(),
                denom: DENOM.into(),
                amount: Uint128::new(88888),
            },
            &[],
        )
        .unwrap();

        Self {
            app,
            bank,
            factory,
            hook_handler,
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

    fn expected_hook_handle_event(&self, from: &str, to: &str, denom: &str, amount: u128) -> Event {
        Event::new("wasm-mock_hook_handle")
            .add_attribute("_contract_addr", &self.hook_handler)
            .add_attribute("from", from.to_string())
            .add_attribute("to", to.to_string())
            .add_attribute("coin", format!("{amount}{}", denom.to_string()))
    }
}

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

#[test]
fn send_hook() {
    let mut suite = TestSuite::setup();

    let res = suite
        .app
        .execute_contract(
            Addr::unchecked("alice"),
            suite.bank.clone(),
            &bank::ExecuteMsg::Send {
                to: "bob".into(),
                coins: coins(12345, DENOM),
            },
            &[],
        )
        .unwrap();

    // the hook message should have been delivered to the handler contract
    // we assert this by checking the events emitted
    let event = res.events.last().cloned().unwrap();
    assert_eq!(event, suite.expected_hook_handle_event("alice", "bob", DENOM, 12345),);

    // alice's balance should have been decreased
    suite.assert_balances("alice", coins(88888 - 12345, DENOM));

    // bob's balance should have been increased
    suite.assert_balances("bob", coins(12345, DENOM));
}

#[test]
fn force_transfer_hook() {
    let mut suite = TestSuite::setup();

    let res = suite
        .app
        .execute_contract(
            Addr::unchecked("pumpkin"),
            suite.factory.clone(),
            &ExecuteMsg::ForceTransfer {
                from: "alice".into(),
                to: "charlie".into(),
                denom: DENOM.into(),
                amount: Uint128::new(22222),
            },
            &[],
        )
        .unwrap();

    let event = res.events.last().cloned().unwrap();
    assert_eq!(event, suite.expected_hook_handle_event("alice", "charlie", DENOM, 22222));

    suite.assert_balances("alice", coins(88888 - 22222, DENOM));
    suite.assert_balances("charlie", coins(22222, DENOM));
}

#[test]
fn sudo_transfer_hook() {
    let mut suite = TestSuite::setup();

    let res = suite
        .app
        .wasm_sudo(
            suite.bank.clone(),
            &bank::SudoMsg::Transfer {
                from: "alice".into(),
                to: "dave".into(),
                coins: coins(33333, DENOM),
            },
        )
        .unwrap();

    let event = res.events.last().cloned().unwrap();
    assert_eq!(event, suite.expected_hook_handle_event("alice", "dave", DENOM, 33333));

    suite.assert_balances("alice", coins(88888 - 33333, DENOM));
    suite.assert_balances("dave", coins(33333, DENOM));
}
