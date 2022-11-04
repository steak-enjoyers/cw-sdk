use cosmwasm_std::{
    from_binary, from_slice, testing::MockQuerier as BaseMockQuerier, to_binary,
    AllBalanceResponse, BalanceResponse, BankQuery, Coin, Empty, Querier, QuerierResult,
    QueryRequest, SupplyResponse, SystemError, WasmQuery,
};

use cw_bank::msg::QueryMsg as BankQueryMsg;

use crate::tests::BANK;

#[derive(Default)]
pub(super) struct MockQuerier {
    base: BaseMockQuerier<Empty>,
}

impl Querier for MockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = from_slice(bin_request)
            .map_err(|err| SystemError::InvalidRequest {
                error: format!("[mock]: parsing query request: {}", err),
                request: bin_request.into(),
            })
            .unwrap();

        match request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg,
            }) => {
                if contract_addr == BANK {
                    if let Ok(msg) = from_binary::<BankQueryMsg>(&msg) {
                        return self.handle_bank_query(msg);
                    }
                }

                panic!("[mock]: unsupported wasm query: {:?}", msg);
            },

            _ => self.base.handle_query(&request),
        }
    }
}

impl MockQuerier {
    pub fn update_balance(
        &mut self,
        addr: impl Into<String>,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        self.base.update_balance(addr, balance)
    }

    fn handle_bank_query(&self, msg: BankQueryMsg) -> QuerierResult {
        match msg {
            BankQueryMsg::Supply {
                denom,
            } => {
                let bin_res = self
                    .base
                    .handle_query(&QueryRequest::Bank(BankQuery::Supply {
                        denom,
                    }))
                    .unwrap()
                    .unwrap();
                let res: SupplyResponse = from_slice(&bin_res).unwrap();
                Ok(to_binary(&res.amount).into()).into()
            },
            BankQueryMsg::Balance {
                address,
                denom,
            } => {
                let bin_res = self
                    .base
                    .handle_query(&QueryRequest::Bank(BankQuery::Balance {
                        address,
                        denom,
                    }))
                    .unwrap()
                    .unwrap();
                let res: BalanceResponse = from_slice(&bin_res).unwrap();
                Ok(to_binary(&res.amount).into()).into()
            },
            BankQueryMsg::Balances {
                address,
                start_after,
                limit,
            } => {
                if start_after.is_some() || limit.is_some() {
                    panic!("[mock]: bank `balances` query does not support pagination");
                }

                let bin_res = self
                    .base
                    .handle_query(&QueryRequest::Bank(BankQuery::AllBalances {
                        address,
                    }))
                    .unwrap()
                    .unwrap();
                let res: AllBalanceResponse = from_slice(&bin_res).unwrap();
                Ok(to_binary(&res.amount).into()).into()
            },
            _ => {
                panic!("[mock]: bank only supports `supply|balance|balances` queries");
            },
        }
    }
}
