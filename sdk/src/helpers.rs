use cosmwasm_std::{Addr, Api, Coin, StdResult};

pub fn stringify_coins(coins: &[Coin]) -> String {
    coins.iter().map(|coin| coin.to_string()).collect::<Vec<_>>().join(",")
}

pub fn stringify_option(opt: Option<impl ToString>) -> String {
    opt.map_or_else(|| "null".to_string(), |value| value.to_string())
}

pub fn validate_optional_addr(api: &dyn Api, opt: Option<&String>) -> StdResult<Option<Addr>> {
    opt.map(|s| api.addr_validate(s)).transpose()
}
