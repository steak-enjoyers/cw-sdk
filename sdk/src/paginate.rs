use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Bound, KeyDeserialize, Map, PrimaryKey};
use serde::{de::DeserializeOwned, ser::Serialize};

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 30;

/// Paginate a Map<K, V> with the given `start` and `limit`, use an enslosure to
/// convert the KV pair to a response type R, and collect to a Vec.
///
/// Inspired by this DAO DAO library:
/// https://github.com/DA0-DA0/dao-contracts/blob/main/packages/cw-paginate/src/lib.rs
pub fn paginate_map<'a, K, D, V, R, E, F>(
    map: Map<'a, K, V>,
    store: &dyn Storage,
    start: Option<Bound<'a, K>>,
    limit: Option<u32>,
    parse_fn: F,
) -> Result<Vec<R>, E>
where
    K: PrimaryKey<'a> + KeyDeserialize<Output = D>,
    V: Serialize + DeserializeOwned,
    D: 'static,
    F: FnMut(StdResult<(D, V)>) -> Result<R, E>,
{
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    map
        .range(store, start, None, Order::Ascending)
        .take(limit)
        .map(parse_fn)
        .collect()
}
