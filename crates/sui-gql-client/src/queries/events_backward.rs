use super::Error;
use crate::queries::model::fragments::{EventEdge, EventFilter};
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

/// Return a single page of events + cursors and a flag indicating if there's a previous page.
///
/// If `page_size` is left `None`, the server decides the size of the page.
///
/// The edges are returned in reverse order of which they where returned by the server
pub async fn query<C: GraphQlClient>(
    client: &C,
    filter: Option<EventFilter>,
    cursor: Option<String>,
    page_size: Option<u32>,
) -> super::Result<(Vec<EventEdge>, bool), C> {
    let vars = Variables {
        filter,
        before: cursor,
        last: page_size.map(|v| v as i32),
    };
    let data: Option<Query> = client
        .query(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    graphql_extract::extract!(data => {
        events? {
            edges
            page_info {
                has_previous_page
            }
        }
    });
    Ok((edges, has_previous_page))
}

// =============================================================================
//  Initial query
// =============================================================================

#[derive(cynic::QueryVariables, Debug, Clone)]
struct Variables {
    last: Option<i32>,
    before: Option<String>,
    filter: Option<EventFilter>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "Variables")]
struct Query {
    #[arguments(before: $before, filter: $filter, last: $last)]
    events: Option<EventConnection>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn init_gql_output() {
    use cynic::QueryBuilder as _;
    let filter = EventFilter {
        sender: None,
        after_checkpoint: None,
        before_checkpoint: None,
        at_checkpoint: None,
        type_: None,
        module: None,
    };
    let vars = Variables {
        filter: Some(filter),
        before: None,
        last: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($last: Int, $before: String, $filter: EventFilter) {
      events(before: $before, filter: $filter, last: $last) {
        edges {
          node {
            timestamp
            contents {
              type {
                repr
              }
              bcs
            }
          }
          cursor
        }
        pageInfo {
          hasPreviousPage
        }
      }
    }
    "###);
}

// =============================================================================
//  Inner query fragments
// =============================================================================

#[derive(cynic::QueryFragment, Clone, Debug)]
struct EventConnection {
    edges: Vec<EventEdge>,
    page_info: HasPreviousPage,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "PageInfo")]
pub struct HasPreviousPage {
    pub has_previous_page: bool,
}
