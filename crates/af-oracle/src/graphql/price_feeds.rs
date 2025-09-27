use af_move_type::MoveInstance;
use af_sui_types::{ObjectId, Version};
use enum_as_inner::EnumAsInner;
use futures::Stream;
use sui_gql_client::queries::fragments::{MoveValueRaw, PageInfoForward};
use sui_gql_client::queries::{Error, GraphQlClientExt as _};
use sui_gql_client::{GraphQlClient, GraphQlResponseExt as _, schema};

type PriceFeed = MoveInstance<crate::oracle::PriceFeed>;

pub(super) fn query<C: GraphQlClient>(
    client: &C,
    pfs: ObjectId,
    version: Option<Version>,
) -> impl Stream<Item = Result<(ObjectId, PriceFeed), Error<C::Error>>> + '_ {
    async_stream::try_stream! {
        let mut vars = Variables {
            pfs,
            version,
            first: Some(client.max_page_size().await?),
            after: None,
        };
        let mut has_next_page = true;
        while has_next_page {
            let (page_info, price_feeds) = request(client, vars.clone()).await?;

            vars.after = page_info.end_cursor.clone();
            has_next_page = page_info.has_next_page;

            for value in price_feeds {
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
        impl Iterator<Item = (ObjectId, PriceFeed)> + 'static,
    ),
    Error<C::Error>,
> {
    let response = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?;
    let data = response.try_into_data()?;

    let PfsDfsConnection { nodes, page_info } = extract(data)?;
    Ok((page_info, nodes.into_iter().filter_map(filter_df)))
}

fn extract(data: Option<Query>) -> Result<PfsDfsConnection, &'static str> {
    graphql_extract::extract!(data => {
        price_feed_storage? {
            dfs
        }
    });
    Ok(dfs)
}

fn filter_df(df: PfsDf) -> Option<(ObjectId, PriceFeed)> {
    let df_name: MoveInstance<crate::keys::PriceFeedForSource> = df.df_name?.try_into().ok()?;
    let df_value_raw = df.df_value?.into_move_value().ok();
    let df_value: PriceFeed = df_value_raw?.try_into().ok()?;

    Some((df_name.value.source_wrapper_id.bytes, df_value))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        pfs: ObjectId::ZERO,
        version: None,
        first: Some(10),
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($pfs: SuiAddress!, $version: UInt53, $first: Int, $after: String) {
      price_feed_storage: object(address: $pfs, version: $version) {
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
    pfs: ObjectId,
    version: Option<Version>,
    first: Option<i32>,
    after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $pfs, version: $version)]
    #[cynic(alias, rename = "object")]
    price_feed_storage: Option<PfsObject>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Object", variables = "Variables")]
struct PfsObject {
    #[arguments(first: $first, after: $after)]
    #[cynic(alias, rename = "dynamicFields")]
    dfs: PfsDfsConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicFieldConnection")]
struct PfsDfsConnection {
    nodes: Vec<PfsDf>,
    page_info: PageInfoForward,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField")]
struct PfsDf {
    #[cynic(alias, rename = "name")]
    df_name: Option<MoveValueRaw>,
    #[cynic(alias, rename = "value")]
    df_value: Option<PfsDfValue>,
}

#[derive(cynic::InlineFragments, Debug, EnumAsInner)]
#[cynic(graphql_type = "DynamicFieldValue")]
enum PfsDfValue {
    MoveValue(MoveValueRaw),
    #[cynic(fallback)]
    Unknown,
}
