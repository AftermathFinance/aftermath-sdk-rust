use af_move_type::MoveInstance;
use af_sui_types::{ObjectId, Version};
use enum_as_inner::EnumAsInner;
use futures::Stream;
use sui_gql_client::queries::fragments::{MoveValueRaw, PageInfoForward};
pub use sui_gql_client::queries::Error;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::{extract, schema, GraphQlClient, GraphQlResponseExt as _};

use crate::orderbook::Order;
use crate::ordered_map::Leaf;

pub(super) fn query<C: GraphQlClient>(
    client: &C,
    map: ObjectId,
    ch_version: Option<Version>,
) -> impl Stream<Item = Result<(u128, Order), Error<C::Error>>> + '_ {
    async_stream::try_stream! {
        let mut vars = Variables {
            map,
            ch_version,
            first: Some(client.max_page_size().await?),
            after: None,
        };
        let mut has_next_page = true;
        while has_next_page {
            let (page_info, orders) = request(client, vars.clone()).await?;

            vars.after = page_info.end_cursor.clone();
            has_next_page = page_info.has_next_page;

            for value in orders {
                yield value;
            }
        }
    }
}

async fn request<C: GraphQlClient>(
    client: &C,
    vars: Variables,
) -> Result<
    (
        PageInfoForward,
        impl Iterator<Item = (u128, Order)> + 'static,
    ),
    Error<C::Error>,
> {
    let response = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?;
    let data = response.try_into_data()?;

    let MapDfsConnection { nodes, page_info } = extract!(data?.map?.map_dfs);
    Ok((page_info, nodes.into_iter().flat_map(MapDf::into_orders)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        map: ObjectId::ZERO,
        ch_version: None,
        first: Some(10),
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($map: SuiAddress!, $chVersion: UInt53, $first: Int, $after: String) {
      map: owner(address: $map, rootVersion: $chVersion) {
        map_dfs: dynamicFields(first: $first, after: $after) {
          nodes {
            map_df: value {
              __typename
              ... on MoveValue {
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
    }
    "###);
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    map: ObjectId,
    ch_version: Option<Version>,
    first: Option<i32>,
    after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $map, rootVersion: $ch_version)]
    #[cynic(alias, rename = "owner")]
    map: Option<MapAsOwner>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Owner", variables = "Variables")]
struct MapAsOwner {
    #[arguments(first: $first, after: $after)]
    #[cynic(alias, rename = "dynamicFields")]
    map_dfs: MapDfsConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicFieldConnection")]
struct MapDfsConnection {
    nodes: Vec<MapDf>,
    page_info: PageInfoForward,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField")]
struct MapDf {
    #[cynic(alias, rename = "value")]
    map_df: Option<MapDfValue>,
}

impl MapDf {
    fn into_orders(self) -> impl Iterator<Item = (u128, Order)> {
        self.map_df
            .into_iter()
            .map(MapDfValue::into_move_value)
            .filter_map(Result::ok)
            .map(MoveInstance::<Leaf<Order>>::try_from)
            // If deser. fails, we just assume it was a `Branch` instead of a `Leaf`
            .filter_map(Result::ok)
            .flat_map(|leaf| Vec::from(leaf.value.keys_vals).into_iter())
            .map(|pair| (pair.key, pair.val))
    }
}

#[derive(cynic::InlineFragments, Debug, EnumAsInner)]
#[cynic(graphql_type = "DynamicFieldValue")]
enum MapDfValue {
    MoveValue(MoveValueRaw),
    #[cynic(fallback)]
    Unknown,
}
