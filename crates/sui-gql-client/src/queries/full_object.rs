use af_sui_types::sui::object::Object;
use af_sui_types::ObjectId;
use cynic::{QueryFragment, QueryVariables};

use super::Error;
use crate::{missing_data, scalars, schema, GraphQlClient, GraphQlResponseExt as _};

/// Get the full [`Object`] contents from the server at a certain version or the latest if not
/// specified.
pub async fn query<C>(
    client: &C,
    object_id: ObjectId,
    version: Option<u64>,
) -> Result<Object, Error<C::Error>>
where
    C: GraphQlClient,
{
    let data = client
        .query::<Query, _>(Variables {
            address: object_id,
            version: version.map(From::from),
        })
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("Null data in response"))?;

    let object = data
        .object
        .ok_or(missing_data!("Object not found"))?
        .bcs
        .ok_or(missing_data!("No object BCS"))?;
    Ok(object.into_inner())
}

#[derive(QueryVariables, Clone, Debug)]
struct Variables {
    address: ObjectId,
    version: Option<scalars::UInt53>,
}

#[derive(QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $address, version: $version)]
    object: Option<GqlObject>,
}

#[derive(QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Object")]
struct GqlObject {
    bcs: Option<scalars::Base64Bcs<Object>>,
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
