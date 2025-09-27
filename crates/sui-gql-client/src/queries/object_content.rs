use af_sui_types::{ObjectId, Version};
use graphql_extract::extract;

use super::Error;
use super::fragments::{MoveObjectContent, MoveValueRaw};
use super::outputs::RawMoveStruct;
use crate::{GraphQlClient, GraphQlResponseExt, schema};

pub async fn query<C: GraphQlClient>(
    client: &C,
    object_id: ObjectId,
    version: Option<u64>,
) -> Result<RawMoveStruct, Error<C::Error>> {
    let vars = Variables {
        address: object_id,
        version,
    };
    let data = client
        .query::<Query, Variables>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    extract!(data => {
        object? {
            as_move_object? {
                contents?
            }
        }
    });
    Ok(contents
        .try_into()
        .expect("Only structs can be top-level objects"))
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
    insta::assert_snapshot!(operation.query, @r###"
    query Query($address: SuiAddress!, $version: UInt53) {
      object(address: $address, version: $version) {
        asMoveObject {
          contents {
            type {
              repr
            }
            bcs
          }
        }
      }
    }
    "###);
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Object")]
struct ObjectContent {
    as_move_object: Option<MoveObjectContent<MoveValueRaw>>,
}
