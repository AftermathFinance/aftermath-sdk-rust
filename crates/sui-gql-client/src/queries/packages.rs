use af_sui_types::Address;

use super::Error;
use super::model::fragments::PageInfoForward;
use crate::queries::model::fragments::MovePackageConnection;
use crate::{GraphQlClient, Paged, PagedResponse, missing_data, schema};

pub async fn query<C: GraphQlClient>(
    client: &C,
    package_id: Address,
) -> Result<Vec<(Address, u64)>, Error<C::Error>> {
    let vars = Variables {
        address: package_id,
        first: None,
        after: None,
    };

    let response: PagedResponse<Query> = client.query_paged(vars).await.map_err(Error::Client)?;
    let (init, pages) = response
        .try_into_data()?
        .ok_or_else(|| missing_data!("No data"))?;

    let pages = pages
        .into_iter()
        .map(|x| {
            x.package_versions
                .ok_or_else(|| missing_data!("No pages data"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut result = vec![];

    for p in init
        .package_versions
        .ok_or_else(|| missing_data!("No init data"))?
        .nodes
        .into_iter()
        .chain(pages.into_iter().flat_map(|p| p.nodes))
    {
        let v = p
            .version
            .ok_or_else(|| missing_data!("No version for package"))?;
        result.push((p.address, v));
    }

    Ok(result)
}

#[derive(cynic::QueryVariables, Clone, Debug)]
pub struct Variables {
    address: Address,
    after: Option<String>,
    first: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
pub struct Query {
    #[arguments(address: $address, first: $first, after: $after)]
    pub package_versions: Option<MovePackageConnection>,
}

impl Paged for Query {
    type Input = Variables;
    type NextPage = Self;
    type NextInput = Variables;

    fn next_variables(&self, mut prev_vars: Self::Input) -> Option<Self::NextInput> {
        if let Some(MovePackageConnection {
            page_info:
                PageInfoForward {
                    has_next_page,
                    end_cursor,
                },
            ..
        }) = &self.package_versions
        {
            if *has_next_page {
                prev_vars.after.clone_from(end_cursor);
                Some(prev_vars)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        address: Address::new(rand::random()).into(),
        first: None,
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($address: SuiAddress!, $after: String, $first: Int) {
      packageVersions(address: $address, first: $first, after: $after) {
        nodes {
          address
          version
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
    "###);
}
