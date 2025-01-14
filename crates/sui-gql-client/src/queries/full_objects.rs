use std::collections::HashMap;

use af_sui_types::{Object, ObjectId};
use itertools::{Either, Itertools as _};

use super::fragments::{ObjectFilter, ObjectKey, PageInfoForward};
use crate::queries::Error;
use crate::{missing_data, scalars, schema, GraphQlClient, Paged};

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    objects: impl IntoIterator<Item = (ObjectId, Option<u64>)> + Send,
    page_size: Option<u32>,
) -> Result<HashMap<ObjectId, Object>, Error<C::Error>> {
    // To keep track of all ids requested.
    let mut requested = vec![];

    let (object_ids, object_keys) = objects
        .into_iter()
        .inspect(|(id, _)| requested.push(*id))
        .partition_map(|(id, v)| {
            v.map_or(Either::Left(id), |n| {
                Either::Right(ObjectKey {
                    object_id: id,
                    version: n.into(),
                })
            })
        });

    let vars = Variables {
        after: None,
        first: page_size.map(|v| v.try_into().unwrap_or(i32::MAX)),
        filter: Some(ObjectFilter {
            object_ids: Some(object_ids),
            object_keys: Some(object_keys),
            ..Default::default()
        }),
    };

    let (init, pages) = client
        .query_paged::<Query>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("Empty response data"))?;

    let mut raw_objs = HashMap::new();
    let init_nodes = init.objects.nodes;
    let page_nodes = pages.into_iter().flat_map(|q| q.objects.nodes);
    for ObjectGql { id, object } in init_nodes.into_iter().chain(page_nodes) {
        let wrapped = object.ok_or(missing_data!("Bcs for object {id}"))?;
        raw_objs.insert(id, wrapped.into_inner());
    }
    // Ensure all requested objects were returned
    for id in requested {
        raw_objs
            .contains_key(&id)
            .then_some(())
            .ok_or(missing_data!("Object version for {id}"))?;
    }

    Ok(raw_objs)
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    filter: Option<ObjectFilter>,
    after: Option<String>,
    first: Option<i32>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(filter: $filter, first: $first, after: $after)]
    objects: ObjectConnection,
}

impl Paged for Query {
    type Input = Variables;

    type NextInput = Variables;

    type NextPage = Self;

    fn next_variables(&self, mut prev_vars: Self::Input) -> Option<Self::NextInput> {
        let PageInfoForward {
            has_next_page,
            end_cursor,
        } = &self.objects.page_info;
        if *has_next_page {
            prev_vars.after.clone_from(end_cursor);
            Some(prev_vars)
        } else {
            None
        }
    }
}

// =============================================================================
//  Inner query fragments
// =============================================================================

/// `ObjectConnection` where the `Object` fragment does take any parameters.
#[derive(cynic::QueryFragment, Clone, Debug)]
struct ObjectConnection {
    nodes: Vec<ObjectGql>,
    page_info: PageInfoForward,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Object")]
struct ObjectGql {
    #[cynic(rename = "address")]
    id: ObjectId,
    #[cynic(rename = "bcs")]
    object: Option<scalars::Base64Bcs<Object>>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() -> color_eyre::Result<()> {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        filter: Some(ObjectFilter {
            object_ids: Some(vec![
                "0x4264c07a42f9d002c1244e43a1f0fa21c49e4a25c7202c597b8476ef6bb57113".parse()?,
                "0x60d1a85f81172a7418206f4b16e1e07e40c91cf58783f63f18a25efc81442dcb".parse()?,
            ]),
            ..Default::default()
        }),
        after: None,
        first: None,
    };

    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query);
    Ok(())
}
