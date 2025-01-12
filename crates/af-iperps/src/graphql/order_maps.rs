use af_sui_types::{Address, ObjectId};
use enum_as_inner::EnumAsInner;
use sui_gql_client::queries::fragments::DynamicFieldName;
use sui_gql_client::queries::Error;
use sui_gql_client::{extract, schema, GraphQlClient, GraphQlResponseExt};

use crate::keys;

/// Object ids of the orderbook and asks/bids maps for a market.
///
/// Just a convenient return type for [`order_maps`].
///
/// [`order_maps`]: super::GraphQlClientExt::order_maps
#[derive(Clone, Copy, Debug)]
pub struct OrderMaps {
    pub orderbook: ObjectId,
    pub asks: ObjectId,
    pub bids: ObjectId,
}

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    package: Address,
    ch: ObjectId,
) -> Result<OrderMaps, Error<C::Error>> {
    let vars = Variables {
        ch,
        orderbook: keys::Orderbook::new()
            .move_instance(package)
            .try_into()
            .expect("BCS-serializable"),
        asks: keys::AsksMap::new()
            .move_instance(package)
            .try_into()
            .expect("BCS-serializable"),
        bids: keys::BidsMap::new()
            .move_instance(package)
            .try_into()
            .expect("BCS-serializable"),
    };
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    let orderbook = extract!(data?
        .ch?
        .orderbook?
        .value?
        .as_variant(OrderbookDofValue::MoveObject));
    Ok(OrderMaps {
        orderbook: orderbook.id,
        asks: extract!(orderbook.asks?.value?.as_variant(MapDofValue::MoveObject)).id,
        bids: extract!(orderbook.bids?.value?.as_variant(MapDofValue::MoveObject)).id,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let package = Address::ZERO;
    let vars = Variables {
        ch: ObjectId::ZERO,
        orderbook: keys::Orderbook::new()
            .move_instance(package)
            .try_into()
            .unwrap(),
        asks: keys::AsksMap::new()
            .move_instance(package)
            .try_into()
            .unwrap(),
        bids: keys::BidsMap::new()
            .move_instance(package)
            .try_into()
            .unwrap(),
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($ch: SuiAddress!, $orderbook: DynamicFieldName!, $asks: DynamicFieldName!, $bids: DynamicFieldName!) {
      ch: object(address: $ch) {
        orderbook: dynamicObjectField(name: $orderbook) {
          value {
            __typename
            ... on MoveObject {
              id: address
              asks: dynamicObjectField(name: $asks) {
                value {
                  __typename
                  ... on MoveObject {
                    id: address
                  }
                }
              }
              bids: dynamicObjectField(name: $bids) {
                value {
                  __typename
                  ... on MoveObject {
                    id: address
                  }
                }
              }
            }
          }
        }
      }
    }
    "###);
}

#[derive(cynic::QueryVariables, Debug)]
struct Variables {
    ch: ObjectId,
    orderbook: DynamicFieldName,
    asks: DynamicFieldName,
    bids: DynamicFieldName,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "Variables")]
struct Query {
    #[arguments(address: $ch)]
    #[cynic(alias, rename = "object")]
    ch: Option<ClearingHouseObject>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Object", variables = "Variables")]
struct ClearingHouseObject {
    #[arguments(name: $orderbook)]
    #[cynic(alias, rename = "dynamicObjectField")]
    orderbook: Option<OrderbookDof>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField", variables = "Variables")]
struct OrderbookDof {
    value: Option<OrderbookDofValue>,
}

#[derive(cynic::InlineFragments, Debug, EnumAsInner)]
#[cynic(graphql_type = "DynamicFieldValue", variables = "Variables")]
enum OrderbookDofValue {
    MoveObject(OrderbookMoveObject),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "MoveObject", variables = "Variables")]
struct OrderbookMoveObject {
    #[cynic(alias, rename = "address")]
    id: ObjectId,
    #[arguments(name: $asks)]
    #[cynic(alias, rename = "dynamicObjectField")]
    asks: Option<MapDof>,
    #[arguments(name: $bids)]
    #[cynic(alias, rename = "dynamicObjectField")]
    bids: Option<MapDof>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField")]
struct MapDof {
    value: Option<MapDofValue>,
}

#[derive(cynic::InlineFragments, Debug, EnumAsInner)]
#[cynic(graphql_type = "DynamicFieldValue")]
enum MapDofValue {
    MoveObject(MapMoveObject),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "MoveObject")]
struct MapMoveObject {
    #[cynic(alias, rename = "address")]
    id: ObjectId,
}
