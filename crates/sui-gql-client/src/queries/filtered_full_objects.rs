use std::collections::HashMap;

use af_sui_types::{Address, Object, ObjectId, TypeTag};

use super::fragments::{ObjectFilterV2, PageInfoForward};
use crate::queries::Error;
use crate::{missing_data, scalars, schema, GraphQlClient, Paged};

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    owner: Option<Address>,
    type_: Option<TypeTag>,
    page_size: Option<u32>,
) -> Result<HashMap<ObjectId, Object>, Error<C::Error>> {
    let filter = ObjectFilterV2 {
        owner,
        type_: type_.map(scalars::TypeTag),
        ..Default::default()
    };
    let vars = Variables {
        after: None,
        first: page_size.map(|v| v.try_into().unwrap_or(i32::MAX)),
        filter: Some(filter),
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

    Ok(raw_objs)
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    filter: Option<ObjectFilterV2>,
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
        filter: None,
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
