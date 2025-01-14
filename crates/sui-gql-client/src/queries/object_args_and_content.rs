use af_sui_types::{ObjectArg, ObjectId, Version};
use sui_gql_schema::{scalars, schema};

use super::fragments::{MoveObjectContent, MoveValueRaw};
use super::object_args::{build_oarg_set_mut, ObjectOwner};
use super::objects_flat;
use super::objects_flat::Variables;
use crate::queries::fragments::ObjectFilter;
use crate::queries::outputs::RawMoveStruct;
use crate::{GraphQlClient, GraphQlErrors, PagedResponse};

type Query = objects_flat::Query<Object>;

#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    #[error(transparent)]
    Client(T),
    #[error(transparent)]
    Server(#[from] GraphQlErrors),
    #[error("No data in object args query response")]
    NoData,
    #[error("Missing data for object: {0}")]
    MissingObject(ObjectId),
}

/// Get a sequence of object args and contents corresponding to `object_ids`, but not
/// necessarily in the same order.
///
/// The `mutable` argument controls whether we want to create mutable [`ObjectArg`]s, if they
/// are of the [`ObjectArg::SharedObject`] variant.
///
/// Fails if any object in the response is missing data.
pub async fn query<C: GraphQlClient>(
    client: &C,
    object_ids: impl IntoIterator<Item = ObjectId> + Send,
    mutable: bool,
    page_size: Option<u32>,
) -> Result<Vec<(ObjectArg, RawMoveStruct)>, Error<C::Error>> {
    let filter = ObjectFilter {
        object_ids: Some(object_ids.into_iter().collect()),
        type_: None,
        owner: None,
        object_keys: None,
    };
    let vars = Variables {
        filter: Some(filter),
        after: None,
        first: page_size.map(|n| n as i32),
    };
    let response: PagedResponse<Query> = client.query_paged(vars).await.map_err(Error::Client)?;
    let Some((init, pages)) = response.try_into_data()? else {
        return Err(Error::NoData);
    };

    let mut result = vec![];
    for Object {
        object_id,
        version,
        digest,
        owner,
        as_move_object,
    } in init
        .objects
        .nodes
        .into_iter()
        .chain(pages.into_iter().flat_map(|p| p.objects.nodes))
    {
        let oarg = build_oarg_set_mut(object_id, version, owner, digest, mutable)
            .ok_or_else(|| Error::MissingObject(object_id))?;
        let content = as_move_object
            .and_then(|c| c.into_content())
            .ok_or_else(|| Error::MissingObject(object_id))?
            .try_into()
            .expect("Only Move structs can be top-level objects");
        result.push((oarg, content));
    }

    Ok(result)
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
    insta::assert_snapshot!(operation.query);
}

// =============================================================================
//  Inner query fragments
// =============================================================================

#[derive(cynic::QueryFragment, Debug)]
struct Object {
    #[cynic(rename = "address")]
    object_id: ObjectId,
    version: Version,
    digest: Option<scalars::Digest>,
    owner: Option<ObjectOwner>,
    as_move_object: Option<MoveObjectContent<MoveValueRaw>>,
}
