use super::Error;
use crate::{missing_data, schema, GraphQlClient, GraphQlResponseExt};

pub(super) async fn query<C: GraphQlClient>(client: &C) -> Result<i32, Error<C::Error>> {
    let max_page_size = client
        .query::<Limits, _>(())
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("No data in response"))?
        .service_config
        .max_page_size;
    Ok(max_page_size)
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
struct Limits {
    service_config: ServiceConfig,
}

#[derive(cynic::QueryFragment, Debug)]
struct ServiceConfig {
    max_page_size: i32,
}

#[cfg(test)]
#[test]
fn query_string() {
    use cynic::QueryBuilder as _;
    use insta::assert_snapshot;
    let op = Limits::build(());
    assert_snapshot!(op.query, @r###"
    query Limits {
      serviceConfig {
        maxPageSize
      }
    }
    "###);
}
