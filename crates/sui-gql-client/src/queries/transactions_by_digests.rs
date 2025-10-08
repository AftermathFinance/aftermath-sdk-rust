use af_sui_types::Transaction;
use itertools::Itertools as _;
use sui_gql_schema::scalars::Base64Bcs;

use crate::queries::Error;
use crate::queries::model::fragments::TransactionGql;
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    keys: Vec<String>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(keys: $keys)]
    multi_get_transactions: Vec<Option<TransactionGql>>,
}

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    digests: impl IntoIterator<Item = String> + Send,
) -> super::Result<Vec<Transaction>, C> {
    let mut digests = digests.into_iter().sorted().dedup().collect_vec();

    let vars = Variables {
        keys: digests.clone(),
    };

    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    graphql_extract::extract!(data => {
        transactions: multi_get_transactions
    });

    let returned = transactions
        .into_iter()
        .flatten()
        .filter_map(|o| o.bcs)
        .map(Base64Bcs::into_inner)
        .inspect(|t| {
            digests
                .iter()
                .position(|k| k == &t.digest().to_string())
                .map(|p| digests.swap_remove(p));
        })
        .collect_vec();

    if !digests.is_empty() {
        return Err(Error::MissingData(format!("Digests {digests:?}")));
    }
    Ok(returned)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() -> color_eyre::Result<()> {
    use cynic::QueryBuilder as _;

    // Variables don't matter, we just need it so taht `Query::build()` compiles
    let vars = Variables { keys: vec![] };

    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r"
    query Query($keys: [String!]!) {
      multiGetTransactions(keys: $keys) {
        digest
        transactionBcs
        effects {
          status
        }
      }
    }
    ");
    Ok(())
}
