use af_sui_types::{ObjectId, Version};

use super::Error;
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    package_id: ObjectId,
    version: Version,
) -> super::Result<ObjectId, C> {
    let vars = Variables {
        address: package_id,
        version: Some(version),
    };
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    graphql_extract::extract!(data => {
        package? {
            address
        }
    });
    Ok(address)
}

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    address: ObjectId,
    version: Option<Version>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $address, version: $version)]
    package: Option<MovePackage>,
}

#[derive(cynic::QueryFragment, Debug)]
struct MovePackage {
    address: ObjectId,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        address: ObjectId::ZERO,
        version: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($address: SuiAddress!, $version: UInt53) {
      package(address: $address, version: $version) {
        address
      }
    }
    "###);
}
