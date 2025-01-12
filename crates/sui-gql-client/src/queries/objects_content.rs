use std::collections::HashMap;

use af_sui_types::ObjectId;
use sui_gql_schema::schema;

use super::fragments::{MoveValueRaw, ObjectFilter};
use super::objects_flat::Variables;
use super::outputs::RawMoveStruct;
use super::{objects_flat, Error};
use crate::{missing_data, GraphQlClient};

pub async fn query<C: GraphQlClient>(
    client: &C,
    object_ids: Vec<ObjectId>,
) -> Result<HashMap<ObjectId, RawMoveStruct>, Error<C::Error>> {
    let vars = Variables {
        after: None,
        first: None,
        filter: Some(ObjectFilter {
            object_ids: Some(object_ids),
            ..Default::default()
        }),
    };
    let (init, mut pages) = client
        .query_paged::<Query>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("Empty response data"))?;
    pages.insert(0, init);

    let mut raw_objs = HashMap::new();
    for object in pages.into_iter().flat_map(|q| q.objects.nodes) {
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
    insta::assert_snapshot!(operation.query);
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct Object {
    #[cynic(rename = "address")]
    object_id: ObjectId,
    as_move_object: Option<super::fragments::MoveObjectContent<MoveValueRaw>>,
}
