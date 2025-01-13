use af_sui_types::TransactionEffects;
use cynic::GraphQlResponse;
use sui_gql_schema::scalars;

use crate::{schema, GraphQlClient, GraphQlErrors, GraphQlResponseExt};

#[derive(cynic::QueryVariables, Clone, Debug)]
pub struct Variables {
    /// [TransactionData] struct that has been BCS-encoded and then Base64-encoded.
    ///
    /// [TransactionData]: af_sui_types::TransactionData
    pub tx_bytes: String,
    /// A list of `flag || signature || pubkey` bytes, Base64-encoded.
    pub signatures: Vec<String>,
}

/// Execute a transaction, committing its effects on chain.
///
/// Waits until the transaction has been finalized on chain to return its transaction digest. If the
/// transaction could not be finalized, returns the errors that prevented it, instead.
#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
pub struct Mutation {
    #[arguments(txBytes: $tx_bytes, signatures: $signatures)]
    pub execute_transaction_block: ExecutionResult,
}

impl Mutation {
    /// Execute a transaction, committing its effects on chain.
    ///
    /// Waits until the transaction has been finalized on chain to return its transaction digest. If the
    /// transaction could not be finalized, returns the errors that prevented it, instead.
    ///
    /// Args:
    /// - `tx_bytes`: [TransactionData] struct that has been BCS-encoded and then Base64-encoded.
    /// - `signatures`: A list of `flag || signature || pubkey` bytes, Base64-encoded.
    ///
    /// [TransactionData]: af_sui_types::TransactionData
    #[allow(clippy::future_not_send)]
    pub async fn execute<Client: GraphQlClient>(
        client: &Client,
        tx_bytes: String,
        signatures: Vec<String>,
    ) -> Result<TransactionEffects, Error<Client::Error>> {
        let result: GraphQlResponse<Self> = client
            .mutation(Variables {
                tx_bytes,
                signatures,
            })
            .await
            .map_err(Error::Client)?;
        let Some(Self {
            execute_transaction_block: ExecutionResult { effects, errors },
        }) = result.try_into_data()?
        else {
            return Err(Error::NoData);
        };

        if let Some(errors) = errors {
            return Err(Error::Execution(errors));
        }

        Ok(effects.bcs.into_inner())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    #[error("No data in transaction GQL response")]
    NoData,
    #[error(transparent)]
    Client(T),
    #[error(transparent)]
    GraphQlResponse(#[from] GraphQlErrors),
    #[error("Executing transaction: {0:?}")]
    Execution(Vec<String>),
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct ExecutionResult {
    /// The effects of the executed transaction.
    pub effects: TransactionBlockEffects,
    /// The errors field captures any errors that occurred during execution
    pub errors: Option<Vec<String>>,
}

/// The effects representing the result of executing a transaction block.
#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct TransactionBlockEffects {
    /// Base64 encoded bcs serialization of the on-chain transaction effects.
    pub bcs: scalars::Base64Bcs<TransactionEffects>,
}
