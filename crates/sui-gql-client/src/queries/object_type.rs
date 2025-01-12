use af_sui_types::{ObjectId, StructTag, TypeTag};

use super::Error;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt};

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    id: ObjectId,
) -> Result<StructTag, Error<C::Error>> {
    let data = client
        .query::<ObjectType, _>(Variables { object_id: id })
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("Any data"))?;
    let TypeTag::Struct(tag) = data
        .object
        .ok_or(missing_data!("Top-level object"))?
        .as_move_object
        // HACK: this is actually a conversion error; the object is a package
        .ok_or(missing_data!("As Move object"))?
        .contents
        .ok_or(missing_data!("Contents"))?
        .type_
        .into()
    else {
        unreachable!("Top-level objects are always structs");
    };

    Ok(*tag)
}

#[cfg(test)]
#[test]
fn gql_string() {
    use cynic::QueryBuilder as _;
    use insta::assert_snapshot;
    let operation = ObjectType::build(Variables {
        object_id: ObjectId::ZERO,
    });
    assert_snapshot!(operation.query, @r###"
    query ObjectType($objectId: SuiAddress!) {
      object(address: $objectId) {
        asMoveObject {
          contents {
            type {
              repr
            }
          }
        }
      }
    }
    "###);
}

#[derive(cynic::QueryVariables, Debug)]
struct Variables {
    object_id: ObjectId,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "Variables")]
struct ObjectType {
    #[arguments(address: $object_id)]
    object: Option<Object>,
}

#[derive(cynic::QueryFragment, Debug)]
struct Object {
    as_move_object: Option<MoveObject>,
}

#[derive(cynic::QueryFragment, Debug)]
struct MoveObject {
    contents: Option<MoveValue>,
}

#[derive(cynic::QueryFragment, Debug)]
struct MoveValue {
    #[cynic(rename = "type")]
    type_: super::fragments::MoveTypeTag,
}
