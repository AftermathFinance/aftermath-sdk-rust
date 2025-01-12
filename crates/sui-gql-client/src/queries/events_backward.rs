use af_sui_types::Address as SuiAddress;
use sui_gql_schema::scalars;

use super::fragments::MoveValueRaw;
use super::Error;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt as _, Paged};

#[derive(cynic::InputObject, Debug, Clone)]
pub struct EventFilter {
    pub sender: Option<SuiAddress>,
    pub transaction_digest: Option<String>,
    pub emitting_module: Option<String>,
    pub event_type: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct EventEdge {
    pub node: Event,
    pub cursor: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Event {
    pub timestamp: Option<scalars::DateTime>,
    pub contents: Option<MoveValueRaw>,
}

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
) -> Result<(Vec<EventEdge>, bool), Error<C::Error>> {
    let vars = Variables {
        filter,
        before: cursor,
        last: page_size.map(|v| v as i32),
    };
    let response: Query = client
        .query(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or_else(|| missing_data!("No data"))?;
    Ok(response.extract())
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
    events: EventConnection,
}

impl Query {
    /// The edges in reverse order of which they where returned by the server + a flag indicating
    /// the existence of older events
    fn extract(self) -> (Vec<EventEdge>, bool) {
        let Self {
            events:
                EventConnection {
                    mut edges,
                    page_info,
                },
        } = self;
        edges.reverse();
        (edges, page_info.has_previous_page)
    }
}

impl Paged for Query {
    type Input = Variables;

    type NextInput = Variables;

    type NextPage = Self;

    fn next_variables(&self, mut prev_vars: Self::Input) -> Option<Self::NextInput> {
        if !self.events.page_info.has_previous_page {
            return None;
        }

        if let Some(edge) = self.events.edges.first() {
            prev_vars.before = Some(edge.cursor.clone());
            Some(prev_vars)
        } else {
            None
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn init_gql_output() {
    use cynic::QueryBuilder as _;
    let filter = EventFilter {
        sender: None,
        transaction_digest: None,
        emitting_module: None,
        event_type: None,
    };
    let vars = Variables {
        filter: Some(filter),
        before: None,
        last: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query);
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
