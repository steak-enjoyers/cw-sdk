use std::str::FromStr;

use serde::{Serialize, de::DeserializeOwned};
use tendermint_rpc::{HttpClient, Client};
use tendermint::abci::Path;
use thiserror::Error;

pub async fn do_abci_query<Q: Serialize, R: Serialize + DeserializeOwned>(
    client: &HttpClient,
    query: Q,
) -> Result<R, QueryError> {
    // serialize the query into binary
    let query_bytes = serde_json::to_vec(&query)?;

    // do query
    // must use "app" path
    let app_path = Path::from_str("app").unwrap();
    let result = client
        .abci_query(Some(app_path), query_bytes, None, false)
        .await?;

    // deserialize the response
    serde_json::from_slice(&result.value).map_err(QueryError::from)
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Rpc(#[from] tendermint_rpc::Error),
}
