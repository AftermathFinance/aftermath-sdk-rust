use af_sui_types::TransactionEffects as TransactionEffectsSdk;
use cynic::GraphQlResponse;
use sui_gql_schema::scalars;

use crate::{GraphQlClient, GraphQlErrors, GraphQlResponseExt, schema};

#[derive(cynic::QueryVariables, Clone, Debug)]
pub struct Variables {
    /// [Transaction] struct that has been BCS-encoded and then Base64-encoded.
    ///
    /// [Transaction]: sui_sdk_types::Transaction
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
    #[arguments(transactionDataBcs: $tx_bytes, signatures: $signatures)]
    pub execute_transaction: ExecutionResult,
}

impl Mutation {
    /// Execute a transaction, committing its effects on chain.
    ///
    /// Waits until the transaction has been finalized on chain to return its transaction digest. If the
    /// transaction could not be finalized, returns the errors that prevented it, instead.
    ///
    /// Args:
    /// - `tx_bytes`: [Transaction] struct that has been BCS-encoded and then Base64-encoded.
    /// - `signatures`: A list of `flag || signature || pubkey` bytes, Base64-encoded.
    ///
    /// [Transaction]: sui_sdk_types::Transaction
    #[allow(clippy::future_not_send)]
    pub async fn execute<Client: GraphQlClient>(
        client: &Client,
        tx_bytes: String,
        signatures: Vec<String>,
    ) -> Result<TransactionEffectsSdk, Error<Client::Error>> {
        let result: GraphQlResponse<Self> = client
            .mutation(Variables {
                tx_bytes,
                signatures,
            })
            .await
            .map_err(Error::Client)?;
        let Some(Self {
            execute_transaction: ExecutionResult { effects, errors },
        }) = result.try_into_data()?
        else {
            return Err(Error::NoData);
        };

        if let Some(errors) = errors {
            return Err(Error::Execution(errors));
        }

        if let Some(eff) = effects {
            if let Some(eff_bcs) = eff.effects_bcs {
                Ok(eff_bcs.try_into()?)
            }
        } else {
            return Err(Error::NoData);
        }
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
    pub effects: Option<TransactionEffects>,
    /// The errors field captures any errors that occurred during execution
    pub errors: Option<Vec<String>>,
}

/// The effects representing the result of executing a transaction block.
#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct TransactionEffects {
    /// Base64 encoded bcs serialization of the on-chain transaction effects.
    pub effects_bcs: Option<scalars::Base64Bcs<TransactionEffectsSdk>>,
}
