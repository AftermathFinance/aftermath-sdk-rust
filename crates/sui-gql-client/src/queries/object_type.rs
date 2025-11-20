use af_sui_types::{Address, StructTag, TypeTag};
use graphql_extract::extract;

use super::Error;
use crate::queries::model::fragments::MoveObject;
use crate::{GraphQlClient, GraphQlResponseExt, schema};

#[derive(cynic::QueryVariables, Debug)]
struct Variables {
    object_id: Address,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "Variables")]
struct Query {
    #[arguments(address: $object_id)]
    object: Option<Object>,
}

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    id: Address,
) -> Result<StructTag, Error<C::Error>> {
    let data = client
        .query::<Query, _>(Variables { object_id: id })
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    extract!(data => {
        object? {
            as_move_object? {
                contents? {
                    type_?
                }
            }
        }
    });
    let TypeTag::Struct(tag) = type_.into() else {
        unreachable!("Top-level objects are always structs");
    };

    Ok(*tag)
}

#[cfg(test)]
#[test]
fn gql_string() {
    use cynic::QueryBuilder as _;
    use insta::assert_snapshot;
    let operation = Query::build(Variables {
        object_id: Address::ZERO,
    });
    assert_snapshot!(operation.query, @r"
    query Query($objectId: SuiAddress!) {
      object(address: $objectId) {
        asMoveObject {
          address
          version
          contents {
            type {
              repr
            }
            bcs
          }
        }
      }
    }
    ");
}

#[derive(cynic::QueryFragment, Debug)]
struct Object {
    as_move_object: Option<MoveObject>,
}
