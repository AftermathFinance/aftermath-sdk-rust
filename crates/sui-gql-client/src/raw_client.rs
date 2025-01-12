use cynic::{GraphQlResponse, Operation};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value as Json;

use crate::GraphQlClient;

/// Client that takes a Cynic operation and return the GraphQL response as JSON.
#[trait_variant::make(Send)]
pub trait RawClient {
    type Error;

    /// Run the operation and return the raw JSON GraphQL response.
    async fn run_graphql_raw<Query, Vars>(
        &self,
        operation: Operation<Query, Vars>,
    ) -> Result<Json, Self::Error>
    where
        Vars: Serialize + Send;
}

#[derive(thiserror::Error, Debug)]
pub enum Error<I> {
    #[error(transparent)]
    Inner(I),

    #[error("Deserializing query: {0}")]
    Json(#[from] serde_json::Error),
}

impl<T> GraphQlClient for T
where
    T: RawClient + Sync,
    T::Error: std::error::Error + Send + 'static,
{
    type Error = Error<T::Error>;

    async fn run_graphql<Query, Vars>(
        &self,
        operation: Operation<Query, Vars>,
    ) -> Result<GraphQlResponse<Query>, Self::Error>
    where
        Vars: Serialize + Send,
        Query: DeserializeOwned + 'static,
    {
        let json = self
            .run_graphql_raw(operation)
            .await
            .map_err(Error::Inner)?;
        Ok(serde_json::from_value(json)?)
    }
}
