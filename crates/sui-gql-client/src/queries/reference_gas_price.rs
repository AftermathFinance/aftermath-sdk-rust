use super::Error;
use crate::scalars::BigInt;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt};

pub(super) async fn query<C: GraphQlClient>(client: &C) -> Result<u64, Error<C::Error>> {
    let query: Query = client
        .query(())
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("No data in response"))?;

    Ok(query
        .epoch
        .ok_or(missing_data!("Epoch not found"))?
        .reference_gas_price
        .ok_or(missing_data!("Epoch reference gas price"))?
        .into_inner())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let operation = Query::build(());
    insta::assert_snapshot!(operation.query);
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct Query {
    epoch: Option<Epoch>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct Epoch {
    reference_gas_price: Option<BigInt<u64>>,
}
