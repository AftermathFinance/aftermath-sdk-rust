use std::collections::HashMap;

use af_sui_types::ObjectId;
use futures::TryStreamExt as _;
use sui_gql_schema::schema;

use super::fragments::{MoveValueRaw, ObjectFilterV2};
use super::objects_flat::Variables;
use super::outputs::RawMoveStruct;
use super::{Error, objects_flat};
use crate::{GraphQlClient, GraphQlResponseExt as _, missing_data};

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    object_ids: Vec<ObjectId>,
) -> super::Result<HashMap<ObjectId, RawMoveStruct>, C> {
    let vars = Variables {
        after: None,
        first: None,
        filter: Some(ObjectFilterV2 {
            object_ids: Some(&object_ids),
            ..Default::default()
        }),
    };

    let mut stream = std::pin::pin!(super::stream::forward(client, vars, request));

    let mut raw_objs = HashMap::new();

    while let Some(object) = stream.try_next().await? {
        let object_id = object.object_id;
        let struct_ = object
            .as_move_object
            .ok_or(missing_data!("Not a Move object"))?
            .contents
            .ok_or(missing_data!("Object contents"))?
            .try_into()
            .expect("Only structs can be top-level objects");
        raw_objs.insert(object_id, struct_);
    }

    Ok(raw_objs)
}

type Query = objects_flat::Query<Object>;

async fn request<C: GraphQlClient>(
    client: &C,
    vars: Variables<'_>,
) -> super::Result<
    super::stream::Page<impl Iterator<Item = super::Result<Object, C>> + 'static + use<C>>,
    C,
> {
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    graphql_extract::extract!(data => {
        objects
    });

    Ok(super::stream::Page {
        info: objects.page_info.into(),
        data: objects.nodes.into_iter().map(Ok),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        filter: None,
        first: None,
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($filter: ObjectFilter, $after: String, $first: Int) {
      objects(filter: $filter, first: $first, after: $after) {
        nodes {
          address
          asMoveObject {
            contents {
              type {
                repr
              }
              bcs
            }
          }
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
    "###);
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct Object {
    #[cynic(rename = "address")]
    object_id: ObjectId,
    as_move_object: Option<super::fragments::MoveObjectContent<MoveValueRaw>>,
}
