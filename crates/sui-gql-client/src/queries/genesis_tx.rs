use af_sui_types::TransactionData;
use cynic::GraphQlResponse;

use super::Error;
use crate::{missing_data, scalars, schema, GraphQlClient, GraphQlResponseExt as _};

pub async fn query<C: GraphQlClient>(client: &C) -> Result<TransactionData, Error<C::Error>> {
    let result: GraphQlResponse<Query> = client
        .query(Variables { id: Some(0) })
        .await
        .map_err(Error::Client)?;
    let wrapped = result
        .try_into_data()?
        .ok_or_else(|| missing_data!("No data"))?
        .epoch
        .ok_or_else(|| missing_data!("epoch"))?
        .transaction_blocks
        .nodes
        .pop()
        .ok_or_else(|| missing_data!("transaction_blocks"))?
        .bcs
        .ok_or_else(|| missing_data!("bcs"))?;
    Ok(wrapped.into_inner())
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    id: Option<af_sui_types::Version>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Query", variables = "Variables")]
struct Query {
    #[arguments(id: $id)]
    epoch: Option<Epoch>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct Epoch {
    #[arguments(first: 1)]
    transaction_blocks: TransactionBlockConnection,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct TransactionBlockConnection {
    nodes: Vec<TransactionBlock>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct TransactionBlock {
    bcs: Option<scalars::Base64Bcs<TransactionData>>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn init_gql_output() {
    use cynic::QueryBuilder as _;
    let operation = Query::build(Variables { id: None });
    insta::assert_snapshot!(operation.query);
}
