use cynic::{GraphQlResponse, QueryFragment};

use super::Error;
use crate::scalars::UInt53;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt as _};

pub async fn query<C: GraphQlClient>(client: &C) -> Result<u64, Error<C::Error>> {
    let curr: GraphQlResponse<Query> = client.query(()).await.map_err(Error::Client)?;
    let curr_epoch_id = curr
        .try_into_data()?
        .ok_or_else(|| missing_data!("No data"))?
        .epoch
        .ok_or_else(|| missing_data!("epoch"))?
        .epoch_id;
    Ok(curr_epoch_id.into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn init_gql_output() {
    use cynic::QueryBuilder as _;
    let operation = Query::build(());
    insta::assert_snapshot!(operation.query);
}

#[derive(QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Query")]
struct Query {
    epoch: Option<Epoch>,
}

#[derive(QueryFragment, Clone, Debug)]
struct Epoch {
    epoch_id: UInt53,
}
