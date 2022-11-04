use cosmwasm_std::coin;

use crate::{
    msg::Config,
    query,
    tests::{setup_test, BANK, OWNER},
};

#[test]
fn proper_instantiation() {
    let deps = setup_test();

    let cfg = query::config(deps.as_ref()).unwrap();
    assert_eq!(
        cfg,
        Config {
            owner: OWNER.into(),
            bank: BANK.into(),
            token_creation_fee: Some(coin(12345, "ujuno"))
        },
    );
}
