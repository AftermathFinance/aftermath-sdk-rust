use cynic::http::CynicReqwestError;
use cynic::serde::Serialize;
use cynic::Operation;
use serde_json::Value as Json;

use crate::RawClient;

/// GraphQL client for Sui using [reqwest] as a backend.
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
        let http_result = self
            .client
            .post(&self.endpoint)
            .json(&operation)
            .send()
            .await;
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
