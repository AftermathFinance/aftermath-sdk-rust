use af_move_type::MoveInstance;
use af_sui_types::{ObjectId, Version};
use enum_as_inner::EnumAsInner;
use futures::Stream;
pub use sui_gql_client::queries::Error;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::queries::fragments::{MoveValueRaw, PageInfoForward};
use sui_gql_client::{GraphQlClient, GraphQlResponseExt as _, schema};

type Position = MoveInstance<crate::position::Position>;

pub(super) fn query<C: GraphQlClient>(
    client: &C,
    ch: ObjectId,
    version: Option<Version>,
) -> impl Stream<Item = Result<(u64, Position), Error<C::Error>>> + '_ {
    async_stream::try_stream! {
        let mut vars = Variables {
            ch,
            version,
            first: Some(client.max_page_size().await?),
            after: None,
        };
        let mut has_next_page = true;
        while has_next_page {
            let (page_info, positions) = request(client, vars.clone()).await?;

            vars.after = page_info.end_cursor.clone();
            has_next_page = page_info.has_next_page;

            for value in positions {
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
        impl Iterator<Item = (u64, Position)> + 'static,
    ),
    Error<C::Error>,
> {
    let response = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?;
    let data = response.try_into_data()?;

    let ChDfsConnection { nodes, page_info } = extract(data)?;
    Ok((page_info, nodes.into_iter().filter_map(filter_df)))
}

fn extract(data: Option<Query>) -> Result<ChDfsConnection, &'static str> {
    graphql_extract::extract!(data => {
        clearing_house? {
            dfs
        }
    });
    Ok(dfs)
}

fn filter_df(df: ChDf) -> Option<(u64, Position)> {
    let df_name: MoveInstance<crate::keys::Position> = df.df_name?.try_into().ok()?;
    let df_value_raw = df.df_value?.into_move_value().ok();
    let df_value: Position = df_value_raw?.try_into().ok()?;

    Some((df_name.value.account_id, df_value))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        ch: ObjectId::ZERO,
        version: None,
        first: Some(10),
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($ch: SuiAddress!, $version: UInt53, $first: Int, $after: String) {
      clearing_house: object(address: $ch, version: $version) {
        dfs: dynamicFields(first: $first, after: $after) {
          nodes {
            df_name: name {
              type {
                repr
              }
              bcs
            }
            df_value: value {
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
    ch: ObjectId,
    version: Option<Version>,
    first: Option<i32>,
    after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $ch, version: $version)]
    #[cynic(alias, rename = "object")]
    clearing_house: Option<ClearingHouseObject>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Object", variables = "Variables")]
struct ClearingHouseObject {
    #[arguments(first: $first, after: $after)]
    #[cynic(alias, rename = "dynamicFields")]
    dfs: ChDfsConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicFieldConnection")]
struct ChDfsConnection {
    nodes: Vec<ChDf>,
    page_info: PageInfoForward,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField")]
struct ChDf {
    #[cynic(alias, rename = "name")]
    df_name: Option<MoveValueRaw>,
    #[cynic(alias, rename = "value")]
    df_value: Option<ChDfValue>,
}

#[derive(cynic::InlineFragments, Debug, EnumAsInner)]
#[cynic(graphql_type = "DynamicFieldValue")]
enum ChDfValue {
    MoveValue(MoveValueRaw),
    #[cynic(fallback)]
    Unknown,
}
