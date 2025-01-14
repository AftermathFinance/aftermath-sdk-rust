use af_sui_types::ObjectId;

use crate::queries::Error;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt as _};

pub async fn query<C>(client: &C, object_id: ObjectId) -> Result<(u64, u64), Error<C::Error>>
where
    C: GraphQlClient,
{
    let data = client
        .query::<Query, _>(Variables { object_id })
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("Null data in response"))?;

    Ok((
        data.checkpoint
            .ok_or(missing_data!("Checkpoint"))?
            .sequence_number,
        data.object
            .ok_or(missing_data!("Object not found"))?
            .version,
    ))
}

#[derive(cynic::QueryVariables, Debug)]
struct Variables {
    object_id: ObjectId,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    checkpoint: Option<Checkpoint>,

    #[arguments(address: $object_id)]
    object: Option<Object>,
}

#[derive(cynic::QueryFragment, Debug)]
struct Object {
    version: af_sui_types::Version,
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

    let vars = Variables {
        object_id: ObjectId::new(rand::random()),
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query);
}
