use cynic::QueryFragment;
use graphql_extract::extract;

use super::Error;
use crate::queries::model::fragments::Epoch;
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

pub async fn query<C: GraphQlClient>(client: &C) -> Result<(u64, u64), Error<C::Error>> {
    let data = client
        .query::<Query, _>(())
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    extract!(data => {
        epoch? {
            epoch_id
            reference_gas_price?
        }
    });
    Ok((epoch_id, reference_gas_price.into_inner()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn init_gql_output() {
    use cynic::QueryBuilder as _;
    let operation = Query::build(());
    insta::assert_snapshot!(operation.query, @r"
    query Query {
      epoch {
        epochId
        referenceGasPrice
      }
    }
    ");
}

#[derive(QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Query")]
struct Query {
    epoch: Option<Epoch>,
}
