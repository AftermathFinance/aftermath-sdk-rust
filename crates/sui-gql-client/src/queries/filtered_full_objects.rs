use af_sui_types::{Address, Object, ObjectId};
use futures::Stream;
use sui_gql_schema::scalars::Base64Bcs;

use super::fragments::{ObjectFilterV2, PageInfo, PageInfoForward};
use super::stream;
use crate::queries::Error;
use crate::{GraphQlClient, GraphQlResponseExt, scalars, schema};

pub(super) fn query<C: GraphQlClient>(
    client: &C,
    owner: Option<Address>,
    type_: Option<String>,
    page_size: Option<u32>,
) -> impl Stream<Item = super::Result<Object, C>> + '_ {
    let filter = ObjectFilterV2 {
        owner,
        type_,
        ..Default::default()
    };
    let vars = Variables {
        after: None,
        first: page_size.map(|v| v.try_into().unwrap_or(i32::MAX)),
        filter: Some(filter),
    };
    stream::forward(client, vars, request)
}

async fn request<C: GraphQlClient>(
    client: &C,
    vars: Variables<'_>,
) -> super::Result<stream::Page<impl Iterator<Item = super::Result<Object, C>> + 'static + use<C>>, C>
{
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    let stream::Page { info, data } = extract(data)?;
    Ok(stream::Page::new(
        info,
        data.map(|r| r.map_err(Error::MissingData)),
    ))
}

fn extract(
    data: Option<Query>,
) -> Result<stream::Page<impl Iterator<Item = Result<Object, String>>>, &'static str> {
    graphql_extract::extract!(data => {
        objects {
            nodes[] {
                id
                object
            }
            page_info
        }
    });
    let nodes = nodes
        .into_iter()
        .map(|r: Result<_, &'static str>| -> Result<_, String> {
            let (id, bcs) = r?;
            bcs.ok_or_else(|| format!("BCS for object {id}"))
                .map(Base64Bcs::into_inner)
        });
    Ok(stream::Page::new(page_info, nodes))
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables<'a> {
    filter: Option<ObjectFilterV2<'a>>,
    after: Option<String>,
    first: Option<i32>,
}

impl stream::UpdatePageInfo for Variables<'_> {
    fn update_page_info(&mut self, info: &PageInfo) {
        self.after.clone_from(&info.end_cursor)
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
    object: Option<scalars::Base64Bcs<Object>>,
}

#[cfg(test)]
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
