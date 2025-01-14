use af_sui_types::Version;
use cynic::{GraphQlResponse, QueryFragment};

use super::Error;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt as _};

pub async fn query<C: GraphQlClient>(client: &C, epoch_id: u64) -> Result<u64, Error<C::Error>> {
    let result: GraphQlResponse<Query> = client
        .query(Variables { id: Some(epoch_id) })
        .await
        .map_err(Error::Client)?;
    Ok(result
        .try_into_data()?
        .ok_or_else(|| missing_data!("No data"))?
        .epoch
        .ok_or_else(|| missing_data!("epoch"))?
        .checkpoints
        .nodes
        .pop()
        .ok_or_else(|| missing_data!("checkpoints"))?
        .sequence_number)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn init_gql_output() {
    use cynic::QueryBuilder as _;
    let operation = Query::build(Variables { id: None });
    insta::assert_snapshot!(operation.query);
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    id: Option<Version>,
}

#[derive(QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Query", variables = "Variables")]
struct Query {
    #[arguments(id: $id)]
    epoch: Option<Epoch>,
}

#[derive(QueryFragment, Clone, Debug)]
struct Epoch {
    #[arguments(last: 1)]
    checkpoints: CheckpointConnection,
}

#[derive(QueryFragment, Clone, Debug)]
struct CheckpointConnection {
    nodes: Vec<Checkpoint>,
}

#[derive(QueryFragment, Clone, Debug)]
struct Checkpoint {
    sequence_number: Version,
}
