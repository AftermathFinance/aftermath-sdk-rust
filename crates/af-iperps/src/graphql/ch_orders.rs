use af_move_type::MoveInstance;
use af_sui_types::{Address, ObjectId, Version};
use enum_as_inner::EnumAsInner;
use futures::Stream;
use sui_gql_client::queries::fragments::{DynamicFieldName, MoveValueRaw, PageInfoForward};
use sui_gql_client::queries::{Error, GraphQlClientExt as _};
use sui_gql_client::{extract, schema, GraphQlClient, GraphQlResponseExt as _};

use crate::orderbook::Order;
use crate::ordered_map::Leaf;

pub(super) fn query<C: GraphQlClient>(
    client: &C,
    package: Address,
    ch: ObjectId,
    version: Option<Version>,
    asks: bool,
) -> impl Stream<Item = Result<(u128, Order), Error<C::Error>>> + '_ {
    let orderbook: DynamicFieldName = crate::keys::Orderbook::new()
        .move_instance(package)
        .try_into()
        .expect("BCS-serializable");

    let map_name: DynamicFieldName = if asks {
        crate::keys::AsksMap::new()
            .move_instance(package)
            .try_into()
            .expect("BCS-serializable")
    } else {
        crate::keys::BidsMap::new()
            .move_instance(package)
            .try_into()
            .expect("BCS-serializable")
    };
    async_stream::try_stream! {
        let mut vars = Variables {
            ch,
            version,
            orderbook,
            map_name,
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
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    let MapDfsConnection { nodes, page_info } = extract!(
        data?
            .clearing_house?
            .orderbook_dof?
            .orderbook?
            .as_variant(OrderbookDofValue::MoveObject)
            .map_dof?
            .map?
            .as_variant(MapDofValue::MoveObject)
            .map_dfs
    );
    Ok((page_info, nodes.into_iter().flat_map(MapDf::into_orders)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let package = Address::ZERO;
    let orderbook: DynamicFieldName = crate::keys::Orderbook::new()
        .move_instance(package)
        .try_into()
        .unwrap();

    let map_name: DynamicFieldName = crate::keys::BidsMap::new()
        .move_instance(package)
        .try_into()
        .unwrap();

    let vars = Variables {
        ch: ObjectId::ZERO,
        version: None,
        orderbook,
        map_name,
        first: Some(10),
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($ch: SuiAddress!, $version: UInt53, $orderbook: DynamicFieldName!, $mapName: DynamicFieldName!, $first: Int, $after: String) {
      clearing_house: object(address: $ch, version: $version) {
        orderbook_dof: dynamicObjectField(name: $orderbook) {
          orderbook: value {
            __typename
            ... on MoveObject {
              map_dof: dynamicObjectField(name: $mapName) {
                map: value {
                  __typename
                  ... on MoveObject {
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
                    __typename
                  }
                }
              }
              __typename
            }
          }
        }
      }
    }
    "###);
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    ch: ObjectId,
    version: Option<Version>,
    orderbook: DynamicFieldName,
    map_name: DynamicFieldName,
    first: Option<i32>,
    after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $ch, version: $version)]
    #[cynic(alias, rename = "object")]
    clearing_house: Option<Object>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Object", variables = "Variables")]
struct Object {
    #[arguments(name: $orderbook)]
    #[cynic(alias, rename = "dynamicObjectField")]
    orderbook_dof: Option<OrderbookDof>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField", variables = "Variables")]
struct OrderbookDof {
    #[cynic(alias, rename = "value")]
    orderbook: Option<OrderbookDofValue>,
}

#[derive(cynic::InlineFragments, Debug, EnumAsInner)]
#[cynic(graphql_type = "DynamicFieldValue", variables = "Variables")]
enum OrderbookDofValue {
    MoveObject(OrderbookObject),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "MoveObject", variables = "Variables")]
struct OrderbookObject {
    #[arguments(name: $map_name)]
    #[cynic(alias, rename = "dynamicObjectField")]
    map_dof: Option<MapDof>,
    __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField", variables = "Variables")]
struct MapDof {
    #[cynic(alias, rename = "value")]
    map: Option<MapDofValue>,
}

#[derive(cynic::InlineFragments, Debug, EnumAsInner)]
#[cynic(graphql_type = "DynamicFieldValue", variables = "Variables")]
enum MapDofValue {
    MoveObject(MapObject),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "MoveObject", variables = "Variables")]
struct MapObject {
    #[arguments(first: $first, after: $after)]
    #[cynic(alias, rename = "dynamicFields")]
    map_dfs: MapDfsConnection,
    __typename: String,
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
            // If deser. fails, we just assume it was a `Branch` instead of a `Leaf`
            .filter_map(|raw| MoveInstance::<Leaf<Order>>::try_from(raw).ok())
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
