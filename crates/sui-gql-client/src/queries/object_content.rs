use af_sui_types::{ObjectId, Version};

use super::fragments::{MoveObjectContent, MoveValueRaw};
use super::outputs::RawMoveStruct;
use super::Error;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt};

pub async fn query<C: GraphQlClient>(
    client: &C,
    object_id: ObjectId,
    version: Option<u64>,
) -> Result<RawMoveStruct, Error<C::Error>> {
    let vars = Variables {
        address: object_id,
        version: version.map(From::from),
    };
    let data = client
        .query::<Query, Variables>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    let out = data
        .ok_or(missing_data!("No data in response"))?
        .object
        .ok_or(missing_data!("Object not found"))?
        .as_move_object
        .ok_or(missing_data!("Not a Move object"))?
        .contents
        .ok_or(missing_data!("Object contents"))?
        .try_into()
        .expect("Only structs can be top-level objects");
    Ok(out)
}

#[derive(cynic::QueryVariables, Debug)]
struct Variables {
    address: ObjectId,
    version: Option<Version>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $address, version: $version)]
    object: Option<ObjectContent>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        address: ObjectId::new(rand::random()),
        version: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query);
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Object")]
struct ObjectContent {
    as_move_object: Option<MoveObjectContent<MoveValueRaw>>,
}
