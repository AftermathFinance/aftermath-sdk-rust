use af_sui_types::{ObjectArg, ObjectId, Version};
use cynic::{QueryFragment, QueryVariables};

use super::Error;
use super::object_args::ObjectOwner;
use crate::{GraphQlClient, GraphQlResponseExt as _, scalars, schema};

/// Get the full [`Object`] contents from the server at a certain version or the latest if not
/// specified.
pub(super) async fn query<C>(client: &C, object_id: ObjectId) -> Result<ObjectArg, Error<C::Error>>
where
    C: GraphQlClient,
{
    let data = client
        .query::<Query, _>(Variables { address: object_id })
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    graphql_extract::extract!(data => {
        object? {
            object_id
            version
            digest
            owner?
        }
    });
    let oarg = super::object_args::build_object_arg_default(object_id, version, owner, digest)
        .ok_or(Error::MissingData("Digest for owned object".into()))?;
    Ok(oarg)
}

#[derive(QueryVariables, Debug)]
struct Variables {
    address: ObjectId,
}

#[derive(QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $address)]
    object: Option<GqlObject>,
}

#[derive(QueryFragment, Debug)]
#[cynic(graphql_type = "Object")]
struct GqlObject {
    #[cynic(rename = "address")]
    object_id: ObjectId,
    version: Version,
    digest: Option<scalars::Digest>,
    owner: Option<ObjectOwner>,
}

#[cfg(test)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        address: ObjectId::ZERO,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($address: SuiAddress!) {
      object(address: $address) {
        address
        version
        digest
        owner {
          __typename
          ... on Immutable {
            _
          }
          ... on Shared {
            __typename
            initialSharedVersion
          }
          ... on Parent {
            __typename
          }
          ... on AddressOwner {
            __typename
          }
        }
      }
    }
    "###);
}
