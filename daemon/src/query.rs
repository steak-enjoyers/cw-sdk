use std::str::FromStr;

use serde::{Serialize, de::DeserializeOwned};
use tendermint_rpc::{HttpClient, Client};
use tendermint::abci::Path;

use crate::DaemonError;

pub async fn do_abci_query<Q: Serialize, R: DeserializeOwned>(
    client: &HttpClient,
    query: Q,
) -> Result<R, DaemonError> {
    // serialize the query into binary
    let query_bytes = serde_json::to_vec(&query)?;

    // do query
    // must use "app" path
    let app_path = Path::from_str("app")?;
    let result = client
        .abci_query(Some(app_path), query_bytes, None, false)
        .await?;

    // deserialize the response
    serde_json::from_slice(&result.value).map_err(DaemonError::from)
}
