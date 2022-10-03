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
        contract: String,
        msg: Binary,
        funds: Vec<Coin>,
    },
    Migrate {
        contract: String,
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
        contract_addr: u64,
    },
    WasmRaw {
        contract_addr: u64,
        key: Vec<u8>,
    },
    WasmSmart {
        contract_addr: u64,
        msg: Binary,
    },
}
