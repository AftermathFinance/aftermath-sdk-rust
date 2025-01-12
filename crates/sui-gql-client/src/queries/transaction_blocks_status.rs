use super::fragments::{PageInfoForward, TransactionBlockFilter};
use crate::queries::Error;
use crate::{extract, missing_data, schema, GraphQlClient, Paged, PagedResponse};

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    transaction_digests: Vec<String>,
) -> Result<impl Iterator<Item = Result<(String, bool), extract::Error>>, Error<C::Error>> {
    let vars = Variables {
        filter: Some(TransactionBlockFilter {
            transaction_ids: Some(transaction_digests),
            ..Default::default()
        }),
        first: None,
        after: None,
    };

    let response: PagedResponse<Query> = client.query_paged(vars).await.map_err(Error::Client)?;
    let (init, pages) = response
        .try_into_data()?
        .ok_or_else(|| missing_data!("No data"))?;

    Ok(init
        .transaction_blocks
        .nodes
        .into_iter()
        .chain(pages.into_iter().flat_map(|p| p.transaction_blocks.nodes))
        .map(digest_and_status))
}

fn digest_and_status(tx_block: TransactionBlock) -> Result<(String, bool), extract::Error> {
    let d = extract!(tx_block.digest?);
    let success = match extract!(tx_block.effects?.status?) {
        ExecutionStatus::Success => true,
        ExecutionStatus::Failure => false,
    };
    Ok((d, success))
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct Variables {
    filter: Option<TransactionBlockFilter>,
    after: Option<String>,
    first: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
#[cynic(variables = "Variables")]
pub struct Query {
    #[arguments(filter: $filter, first: $first, after: $after)]
    pub transaction_blocks: TransactionBlockConnection,
}

impl Paged for Query {
    type Input = Variables;
    type NextPage = Self;
    type NextInput = Variables;

    fn next_variables(&self, mut prev_vars: Self::Input) -> Option<Self::NextInput> {
        let PageInfoForward {
            has_next_page,
            end_cursor,
        } = &self.transaction_blocks.page_info;
        if *has_next_page {
            prev_vars.after.clone_from(end_cursor);
            Some(prev_vars)
        } else {
            None
        }
    }
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
