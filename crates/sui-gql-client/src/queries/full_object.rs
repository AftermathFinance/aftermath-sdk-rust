use af_sui_types::{Object, ObjectId};
use cynic::{QueryFragment, QueryVariables};
use graphql_extract::extract;

use super::Error;
use crate::{GraphQlClient, GraphQlResponseExt as _, scalars, schema};

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
            version,
        })
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    extract!(data => {
        object? {
            bcs?
        }
    });
    Ok(bcs.into_inner())
}

#[derive(QueryVariables, Clone, Debug)]
struct Variables {
    address: ObjectId,
    version: Option<af_sui_types::Version>,
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
    insta::assert_snapshot!(operation.query, @r###"
    query Query($address: SuiAddress!, $version: UInt53) {
      object(address: $address, version: $version) {
        bcs
      }
    }
    "###);
}
