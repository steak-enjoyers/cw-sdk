use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin};

#[cw_serde]
pub enum SdkMsg {
    StoreCode {
        wasm_byte_code: Binary,
    },
    Instantiate {
        code_id: u64,
        msg: Binary,
    },
    Execute {
        contract: u64,
        msg: Binary,
        funds: Vec<Coin>,
    },
    Migrate {
        contract: u64,
        code_id: u64,
        msg: Binary,
    },
}

#[cw_serde]
pub enum SdkQuery {
    Code {
        code_id: u64,
    },
    Contract {
        contract: u64,
    },
    WasmRaw {
        contract: u64,
        key: Binary,
    },
    WasmSmart {
        contract: u64,
        msg: Binary,
    },
}
