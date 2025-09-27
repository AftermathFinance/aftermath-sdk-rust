use std::collections::HashMap;

use af_sui_types::{Address, ObjectId, Version};
use graphql_extract::extract;

use super::fragments::PageInfoForward;
use crate::queries::Error;
use crate::{GraphQlClient, GraphQlResponseExt, missing_data, scalars, schema};

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables<'a> {
    filter: Option<ObjectFilter<'a>>,
    after: Option<String>,
    first: Option<i32>,
}

impl Variables<'_> {
    fn next_variables(mut self, page_info: &PageInfoForward) -> Option<Self> {
        let PageInfoForward {
            has_next_page,
            end_cursor,
        } = page_info;
        if *has_next_page {
            self.after.clone_from(end_cursor);
            Some(self)
        } else {
            None
        }
    }
}

#[derive(cynic::InputObject, Clone, Debug, Default)]
struct ObjectFilter<'a> {
    #[cynic(rename = "type")]
    type_: Option<&'a scalars::TypeTag>,
    owner: Option<&'a Address>,
    object_ids: Option<&'a [ObjectId]>,
}

pub async fn query<C: GraphQlClient>(
    client: &C,
    object_ids: &[ObjectId],
) -> super::Result<(u64, HashMap<ObjectId, u64>), C> {
    let vars = Variables {
        after: None,
        first: None,
        filter: Some(ObjectFilter {
            object_ids: Some(object_ids),
            ..Default::default()
        }),
    };
    let init = client
        .query::<Query, _>(vars.clone())
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    extract!(init => {
        checkpoint? {
            sequence_number
        }
        objects {
            nodes
            page_info
        }
    });
    let ckpt_num = sequence_number;
    let init_nodes = nodes;

    let mut next_vars = vars.next_variables(&page_info);
    let mut pages = vec![];
    while let Some(vars) = next_vars {
        let Some(next_page) = client
            .query::<QueryPage, _>(vars.clone())
            .await
            .map_err(Error::Client)?
            .try_into_data()?
        else {
            break;
        };
        next_vars = vars.next_variables(&next_page.objects.page_info);
        pages.push(next_page);
    }

    let mut raw_objs = HashMap::new();
    let page_nodes = pages.into_iter().flat_map(|q| q.objects.nodes);
    for object in init_nodes.into_iter().chain(page_nodes) {
        let object_id = object.object_id;
        let version = object.version;
        raw_objs.insert(object_id, version);
    }
    // Ensure all input objects have versions
    for id in object_ids {
        raw_objs
            .contains_key(id)
            .then_some(())
            .ok_or(missing_data!("Object version for {id}"))?;
    }

    Ok((ckpt_num, raw_objs))
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    checkpoint: Option<Checkpoint>,

    #[arguments(filter: $filter, first: $first, after: $after)]
    objects: ObjectConnection,
}

// =============================================================================
//  Subsequent pages
// =============================================================================

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "Variables")]
struct QueryPage {
    #[arguments(filter: $filter, first: $first, after: $after)]
    objects: ObjectConnection,
}

// =============================================================================
//  Inner query fragments
// =============================================================================

#[derive(cynic::QueryFragment, Debug, Clone)]
struct Object {
    version: Version,
    #[cynic(rename = "address")]
    object_id: ObjectId,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
struct Checkpoint {
    sequence_number: Version,
}

/// `ObjectConnection` where the `Object` fragment does take any parameters.
#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "ObjectConnection")]
struct ObjectConnection {
    nodes: Vec<Object>,
    page_info: PageInfoForward,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let ids = vec![
        "0x4264c07a42f9d002c1244e43a1f0fa21c49e4a25c7202c597b8476ef6bb57113"
            .parse()
            .unwrap(),
        "0x60d1a85f81172a7418206f4b16e1e07e40c91cf58783f63f18a25efc81442dcb"
            .parse()
            .unwrap(),
    ];

    let vars = Variables {
        filter: Some(ObjectFilter {
            object_ids: Some(&ids),
            ..Default::default()
        }),
        after: None,
        first: None,
    };

    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($filter: ObjectFilter, $after: String, $first: Int) {
      checkpoint {
        sequenceNumber
      }
      objects(filter: $filter, first: $first, after: $after) {
        nodes {
          version
          address
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
    "###);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn page_gql_output() {
    use cynic::QueryBuilder as _;
    let ids = vec![
        "0x4264c07a42f9d002c1244e43a1f0fa21c49e4a25c7202c597b8476ef6bb57113"
            .parse()
            .unwrap(),
        "0x60d1a85f81172a7418206f4b16e1e07e40c91cf58783f63f18a25efc81442dcb"
            .parse()
            .unwrap(),
    ];
    let vars = Variables {
        filter: Some(ObjectFilter {
            object_ids: Some(&ids),
            ..Default::default()
        }),
        after: None,
        first: None,
    };
    let operation = QueryPage::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query QueryPage($filter: ObjectFilter, $after: String, $first: Int) {
      objects(filter: $filter, first: $first, after: $after) {
        nodes {
          version
          address
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
    "###);
}
