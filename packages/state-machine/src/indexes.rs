use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList};

use crate::Account;

/// The index types used to index accounts in cw-sdk
pub struct AccountIndexes<'a> {
    /// Index accounts by contract labels. If an account is a base account
    /// then it is not indexed.
    pub label: OptionalUniqueIndex<'a, String, Account<Addr>, &'a Addr>,
}

impl<'a> AccountIndexes<'a> {
    pub const fn new(label_namespace: &'a str) -> Self {
        Self {
            label: OptionalUniqueIndex::new(
                |account| match account {
                    Account::Base {
                        ..
                    } => None,
                    Account::Contract {
                        label,
                        ..
                    } => Some(label.clone()),
                },
                label_namespace,
            ),
        }
    }
}

impl<'a> IndexList<Account<Addr>> for AccountIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Account<Addr>>> + '_> {
        let v: Vec<&dyn Index<Account<Addr>>> = vec![&self.label];
        Box::new(v.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MockStorage;
    use cw_storage_plus::IndexedMap;

    use super::*;

    const ACCOUNTS: IndexedMap<&Addr, Account<Addr>, AccountIndexes> = IndexedMap::new(
        "accounts",
        AccountIndexes::new("accounts_label"),
    );

    #[test]
    fn indexing_accounts() {
        let mut store = MockStorage::new();

        let addresses = ["base1", "base2", "bank", "token-factory"]
            .into_iter()
            .map(Addr::unchecked)
            .collect::<Vec<_>>();

        let accounts = [
            Account::Base {
                pubkey: b"base1pubkey".into(),
                sequence: 0,
            },
            Account::Base {
                pubkey: b"base2pubkey".into(),
                sequence: 123,
            },
            Account::Contract {
                code_id: 234,
                label: "bank".into(),
                admin: None,
            },
            Account::Contract {
                code_id: 345,
                label: "token-factory".into(),
                admin: Some(Addr::unchecked("larry")),
            },
        ];

        addresses
            .iter()
            .zip(accounts.iter())
            .try_for_each(|(addr, acct)| ACCOUNTS.save(&mut store, addr, acct))
            .unwrap();

        // bank contract should have been indexed
        let (addr, acct) = ACCOUNTS
            .idx
            .label
            .may_load(&store, "bank".into())
            .unwrap()
            .unwrap();
        assert_eq!(addr, addresses[2]);
        assert_eq!(acct, accounts[2]);

        // token-factory contract should have been indexed
        let (addr, acct) = ACCOUNTS
            .idx
            .label
            .may_load(&store, "token-factory".into())
            .unwrap()
            .unwrap();
        assert_eq!(addr, addresses[3]);
        assert_eq!(acct, accounts[3]);

        // the base accounts should not have been indexed
        // meaning the total number of entries in the idx_map should be 2
        let items = ACCOUNTS
            .idx
            .label
            .range(&store, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(items.len(), 2)
    }

    #[test]
    fn rejecting_duplicate_indexes() {
        let mut store = MockStorage::new();

        // store the account for the first time, should succeed
        let addr = Addr::unchecked("bank");
        let acct = Account::Contract {
            code_id: 234,
            label: "bank".into(),
            admin: None,
        };
        ACCOUNTS.save(&mut store, &addr, &acct).unwrap();

        // !!! IMPORTANT !!!
        // if we write to the *same key* with the same index, there will not be
        // a duplicate index error.
        // the duplicate error is only raised when there are two *different keys*
        // with the same index.
        // therefore, when instantiating contracts, we must assert that a contract
        // with the same address/label does not already exist!
        let acct = Account::Contract {
            code_id: 42069,
            label: "bank".into(), // same label but different code id and admin
            admin: Some(Addr::unchecked("jake")),
        };
        let res = ACCOUNTS.save(&mut store, &addr, &acct);
        assert!(res.is_ok());

        // store another account with the same label, should fail
        let addr = Addr::unchecked("token-factory");
        let acct = Account::Contract {
            code_id: 345,
            label: "bank".into(), // pretend we type the wrong label by mistake; should be `token-factory`
            admin: None,
        };
        let err = ACCOUNTS.save(&mut store, &addr, &acct).unwrap_err();
        assert_eq!(err, StdError::generic_err("Violates unique constraint on index"));
    }
}
