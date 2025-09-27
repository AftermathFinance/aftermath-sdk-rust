use std::collections::HashMap;

use af_sui_types::{Object, ObjectId};
use futures::{StreamExt as _, TryStreamExt as _};
use graphql_extract::extract;
use itertools::Itertools as _;
use sui_gql_schema::scalars::Base64Bcs;

use super::fragments::{ObjectFilterV2, PageInfo, PageInfoForward};
use super::stream;
use crate::queries::Error;
use crate::{GraphQlClient, GraphQlResponseExt as _, missing_data, schema};

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    objects: impl IntoIterator<Item = ObjectId> + Send,
    page_size: Option<u32>,
) -> Result<HashMap<ObjectId, Object>, Error<C::Error>> {
    // To keep track of all ids requested.
    let object_ids = objects.into_iter().collect_vec();

    let filter = ObjectFilterV2 {
        object_ids: Some(&object_ids),
        ..Default::default()
    };
    let vars = Variables {
        after: None,
        first: page_size.map(|v| v.try_into().unwrap_or(i32::MAX)),
        filter: Some(filter),
    };

    let raw_objs: HashMap<_, _> = stream::forward(client, vars, request)
        .map(|r| -> super::Result<_, C> {
            let (id, obj) = r?;
            Ok((id, obj.ok_or_else(|| missing_data!("BCS for {id}"))?))
        })
        .try_collect()
        .await?;

    // Ensure all requested objects were returned
    for id in object_ids {
        raw_objs
            .contains_key(&id)
            .then_some(())
            .ok_or(missing_data!("Object {id}"))?;
    }

    Ok(raw_objs)
}

async fn request<C: GraphQlClient>(
    client: &C,
    vars: Variables<'_>,
) -> super::Result<
    stream::Page<
        impl Iterator<Item = super::Result<(ObjectId, Option<Object>), C>> + 'static + use<C>,
    >,
    C,
> {
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    extract!(data => {
        objects {
            nodes[] {
                id
                object
            }
            page_info
        }
    });
    Ok(stream::Page::new(
        page_info,
        nodes.map(|r| -> super::Result<_, C> {
            let (id, obj) = r?;
            Ok((id, obj.map(Base64Bcs::into_inner)))
        }),
    ))
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables<'a> {
    filter: Option<ObjectFilterV2<'a>>,
    after: Option<String>,
    first: Option<i32>,
}

impl stream::UpdatePageInfo for Variables<'_> {
    fn update_page_info(&mut self, info: &PageInfo) {
        self.after.clone_from(&info.end_cursor);
    }
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(filter: $filter, first: $first, after: $after)]
    objects: ObjectConnection,
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
    object: Option<Base64Bcs<Object>>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() -> color_eyre::Result<()> {
    use cynic::QueryBuilder as _;

    // Variables don't matter, we just need it so taht `Query::build()` compiles
    let vars = Variables {
        filter: Some(Default::default()),
        after: None,
        first: None,
    };

    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($filter: ObjectFilter, $after: String, $first: Int) {
      objects(filter: $filter, first: $first, after: $after) {
        nodes {
          address
          bcs
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
    "###);
    Ok(())
}
