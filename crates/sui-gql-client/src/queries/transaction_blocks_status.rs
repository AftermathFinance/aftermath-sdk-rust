use futures::StreamExt as _;

use self::stream::UpdatePageInfo;
use super::fragments::{PageInfoForward, TransactionFilter};
use super::stream;
use crate::queries::Error;
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

type Item = (String, bool);

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct Variables {
    filter: Option<TransactionFilter>,
    after: Option<String>,
    first: Option<i32>,
}

#[expect(deprecated, reason = "Internal `extract` module deprecated")]
pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    transaction_digests: Vec<String>,
) -> super::Result<impl Iterator<Item = Result<Item, crate::extract::Error>> + use<C>, C> {
    let filter = TransactionFilter {
        transaction_ids: Some(transaction_digests),
        ..Default::default()
    };
    let vars = Variables {
        filter: Some(&filter),
        first: None,
        after: None,
    };

    let mut vec = vec![];
    let mut stream = std::pin::pin!(stream::forward(client, vars, request));
    while let Some(res) = stream.next().await {
        match res {
            Ok(item) => vec.push(Ok(item)),
            Err(Error::MissingData(err)) => vec.push(Err(crate::extract::Error::new(err))),
            Err(other) => return Err(other),
        }
    }

    Ok(vec.into_iter())
}

async fn request<C: GraphQlClient>(
    client: &C,
    vars: Variables<'_>,
) -> super::Result<stream::Page<impl Iterator<Item = super::Result<Item, C>> + use<C>>, C> {
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    graphql_extract::extract!(data => {
        transaction_blocks {
            page_info
            nodes[] {
                digest?
                effects? {
                    status?
                }
            }
        }
    });

    Ok(stream::Page::new(
        page_info,
        nodes.map(|r| r.map(|(d, s)| (d, s.into())).map_err(super::Error::from)),
    ))
}

impl UpdatePageInfo for Variables<'_> {
    fn update_page_info(&mut self, info: &super::fragments::PageInfo) {
        self.after.clone_from(&info.end_cursor)
    }
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
#[cynic(variables = "Variables")]
pub struct Query {
    #[arguments(filter: $filter, first: $first, after: $after)]
    pub transaction_blocks: TransactionBlockConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TransactionBlockConnection {
    pub page_info: PageInfoForward,
    pub nodes: Vec<TransactionBlock>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TransactionBlock {
    digest: Option<String>,
    effects: Option<TransactionBlockEffects>,
}

#[derive(cynic::QueryFragment, Debug)]
struct TransactionBlockEffects {
    status: Option<ExecutionStatus>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum ExecutionStatus {
    Success,
    Failure,
}

impl From<ExecutionStatus> for bool {
    fn from(value: ExecutionStatus) -> Self {
        match value {
            ExecutionStatus::Success => true,
            ExecutionStatus::Failure => false,
        }
    }
}

#[cfg(test)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        filter: None,
        after: None,
        first: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($filter: TransactionBlockFilter, $after: String, $first: Int) {
      transactionBlocks(filter: $filter, first: $first, after: $after) {
        pageInfo {
          hasNextPage
          endCursor
        }
        nodes {
          digest
          effects {
            status
          }
        }
      }
    }
    "###);
}
