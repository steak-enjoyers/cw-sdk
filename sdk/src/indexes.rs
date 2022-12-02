use std::marker::PhantomData;

use cosmwasm_std::{Binary, Order, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, Index, KeyDeserialize, Map, PrimaryKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct UniqueRef<T> {
    pk: Binary,
    value: T,
}

/// Similar to `UniqueIndex`, but the index function returns an _optional_ index
/// key. Only saves an entry in the index map if it is `Some`.
///
/// In cw-sdk, this is used in the `ACCOUNTS` map, where smart contract accounts
/// are indexed by their labels such that we can enforce that the labels are
/// unique, while base accounts are not indexed.
pub struct OptionalUniqueIndex<'a, IK, T, PK = ()> {
    index: fn(&T) -> Option<IK>,
    pub(crate) idx_map: Map<'a, IK, UniqueRef<T>>,
    phantom: PhantomData<PK>,
}

impl<'a, IK, T, PK> OptionalUniqueIndex<'a, IK, T, PK> {
    pub const fn new(idx_fn: fn(&T) -> Option<IK>, idx_namespace: &'a str) -> Self {
        Self {
            index: idx_fn,
            idx_map: Map::new(idx_namespace),
            phantom: PhantomData,
        }
    }
}

impl<'a, IK, T, PK> OptionalUniqueIndex<'a, IK, T, PK>
where
    PK: KeyDeserialize,
    IK: PrimaryKey<'a>,
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn may_load(&self, store: &dyn Storage, key: IK) -> StdResult<Option<(PK::Output, T)>> {
        match self.idx_map.may_load(store, key)? {
            Some(UniqueRef {
                pk,
                value,
            }) => {
                let key = PK::from_slice(&pk)?;
                Ok(Some((key, value)))
            },
            None => Ok(None),
        }
    }

    pub fn range<'c>(
        &self,
        store: &'c dyn Storage,
        min: Option<Bound<'a, IK>>,
        max: Option<Bound<'a, IK>>,
        order: Order,
    ) -> Box<dyn Iterator<Item = StdResult<(PK::Output, T)>> + 'c>
    where
        T: 'c,
    {
        let iter = self
            .idx_map
            .range_raw(store, min, max, order)
            .map(|res| {
                let (_, item) = res?;
                let key = PK::from_slice(&item.pk)?;
                Ok((key, item.value))
            });
        Box::new(iter)
    }
}

impl<'a, IK, T, PK> Index<T> for OptionalUniqueIndex<'a, IK, T, PK>
where
    T: Serialize + DeserializeOwned + Clone,
    IK: PrimaryKey<'a>,
{
    fn save(&self, store: &mut dyn Storage, pk: &[u8], data: &T) -> StdResult<()> {
        // only save data in idx_map if the index in `Some`
        if let Some(idx) = (self.index)(data) {
            self.idx_map.update(store, idx, |opt| {
                if opt.is_some() {
                    // TODO: return a more informative error message,
                    // e.g. what the index and associated primary keys are
                    return Err(StdError::generic_err("Violates unique constraint on index"));
                }
                Ok(UniqueRef {
                    pk: pk.into(),
                    value: data.clone(),
                })
            })?;
        }
        Ok(())
    }

    fn remove(&self, store: &mut dyn Storage, _pk: &[u8], old_data: &T) -> StdResult<()> {
        if let Some(idx) = (self.index)(old_data) {
            self.idx_map.remove(store, idx);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::MockStorage, Addr, Order, StdError, StdResult};
    use cw_storage_plus::{Index, IndexList, IndexedMap};

    use crate::Account;

    use super::OptionalUniqueIndex;

    struct AccountIndexes<'a> {
        /// Index accounts by contract labels. If an account is a base account
        /// then it is not indexed.
        pub label: OptionalUniqueIndex<'a, String, Account<Addr>, &'a Addr>,
    }

    impl<'a> IndexList<Account<Addr>> for AccountIndexes<'a> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Account<Addr>>> + '_> {
            let v: Vec<&dyn Index<Account<Addr>>> = vec![&self.label];
            Box::new(v.into_iter())
        }
    }

    const ACCOUNTS: IndexedMap<&Addr, Account<Addr>, AccountIndexes> = IndexedMap::new(
        "accounts",
        AccountIndexes {
            label: OptionalUniqueIndex::new(
                |account: &Account<Addr>| match account {
                    Account::Base {
                        ..
                    } => None,
                    Account::Contract {
                        label,
                        ..
                    } => Some(label.clone()),
                },
                "accounts__label",
            ),
        },
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

        let addr = Addr::unchecked("bank");
        let acct = Account::Contract {
            code_id: 234,
            label: "bank".into(),
            admin: None,
        };

        // store the account for the first time, should succeed
        ACCOUNTS.save(&mut store, &addr, &acct).unwrap();

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
