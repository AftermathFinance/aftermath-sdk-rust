use graphql_extract::extract;

use crate::queries::Error;
use crate::queries::model::fragments::Checkpoint;
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

pub async fn query<C>(client: &C) -> Result<u64, Error<C::Error>>
where
    C: GraphQlClient,
{
    let data = client
        .query::<Query, _>(Variables {})
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    extract!(data => {
        checkpoint? {
            sequence_number
        }
    });

    Ok(sequence_number)
}

#[derive(cynic::QueryVariables, Debug)]
struct Variables {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    checkpoint: Option<Checkpoint>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {};
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query {
      checkpoint {
        sequenceNumber
      }
    }
    "###);
}
