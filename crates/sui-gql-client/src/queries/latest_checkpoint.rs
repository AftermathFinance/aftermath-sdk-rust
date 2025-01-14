use crate::queries::Error;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt as _};

pub async fn query<C>(client: &C) -> Result<u64, Error<C::Error>>
where
    C: GraphQlClient,
{
    let data = client
        .query::<Query, _>(Variables {})
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("Null data in response"))?;

    Ok(data
        .checkpoint
        .ok_or(missing_data!("Checkpoint"))?
        .sequence_number)
}

#[derive(cynic::QueryVariables, Debug)]
struct Variables {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    checkpoint: Option<Checkpoint>,
}

#[derive(cynic::QueryFragment, Debug)]
struct Checkpoint {
    sequence_number: af_sui_types::Version,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {};
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query);
}
