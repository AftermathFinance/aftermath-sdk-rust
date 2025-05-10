use cynic::Operation;
use cynic::http::CynicReqwestError;
use cynic::serde::Serialize;
use serde_json::Value as Json;

use crate::RawClient;

/// GraphQL client for Sui using [reqwest] as a backend.
///
/// This owns the inner HTTP client and endpoint string.
#[derive(Clone, Debug)]
pub struct ReqwestClient {
    client: reqwest::Client,
    endpoint: String,
}

impl ReqwestClient {
    pub const fn new(client: reqwest::Client, endpoint: String) -> Self {
        Self { client, endpoint }
    }

    /// Construct with default inner HTTP client.
    pub fn new_default(endpoint: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into(),
        }
    }
}

impl RawClient for ReqwestClient {
    type Error = CynicReqwestError;

    async fn run_graphql_raw<Query, Vars>(
        &self,
        operation: Operation<Query, Vars>,
    ) -> Result<Json, Self::Error>
    where
        Vars: Serialize + Send,
    {
        self.client
            .graphql(&self.endpoint)
            .run_graphql_raw(operation)
            .await
    }
}

#[extension_traits::extension(pub trait ReqwestExt)]
impl reqwest::Client {
    fn graphql<'a>(&'a self, endpoint: &'a str) -> ReqwestClientRef<'a> {
        ReqwestClientRef {
            inner: self,
            endpoint,
        }
    }
}

/// GraphQL client for Sui using [reqwest] as a backend.
pub struct ReqwestClientRef<'a> {
    inner: &'a reqwest::Client,
    endpoint: &'a str,
}

impl RawClient for ReqwestClientRef<'_> {
    type Error = CynicReqwestError;

    async fn run_graphql_raw<Query, Vars>(
        &self,
        operation: Operation<Query, Vars>,
    ) -> Result<Json, Self::Error>
    where
        Vars: Serialize + Send,
    {
        let http_result = self.inner.post(self.endpoint).json(&operation).send().await;
        deser_gql(http_result).await
    }
}

async fn deser_gql(
    response: Result<reqwest::Response, reqwest::Error>,
) -> Result<Json, CynicReqwestError> {
    let response = match response {
        Ok(response) => response,
        Err(e) => return Err(CynicReqwestError::ReqwestError(e)),
    };

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await;
        let text = match text {
            Ok(text) => text,
            Err(e) => return Err(CynicReqwestError::ReqwestError(e)),
        };

        let Ok(deserred) = serde_json::from_str(&text) else {
            let response = CynicReqwestError::ErrorResponse(status, text);
            return Err(response);
        };

        Ok(deserred)
    } else {
        let json = response.json().await;
        json.map_err(CynicReqwestError::ReqwestError)
    }
}
