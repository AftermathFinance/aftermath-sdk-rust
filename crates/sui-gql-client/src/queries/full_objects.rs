use af_sui_types::{Address, Object, Version};
use itertools::Itertools as _;
use sui_gql_schema::scalars::Base64Bcs;

use super::model::fragments::ObjectKey;
use crate::queries::Error;
use crate::queries::model::fragments::ObjectGql;
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables<'a> {
    keys: &'a [ObjectKey],
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(keys: $keys)]
    multi_get_objects: Vec<Option<ObjectGql>>,
}

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    objects: impl IntoIterator<Item = (Address, Option<Version>)> + Send,
    at_checkpoint: Option<u64>,
) -> super::Result<Vec<Object>, C> {
    let mut object_keys = objects
        .into_iter()
        .sorted()
        .dedup()
        .map(|(object_id, version)| ObjectKey {
            address: object_id,
            version,
            root_version: None,
            at_checkpoint,
        })
        .collect_vec();

    let vars = Variables { keys: &object_keys };

    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    graphql_extract::extract!(data => {
        objects: multi_get_objects
    });

    let returned = objects
        .into_iter()
        .flatten()
        .filter_map(|o| o.object)
        .map(Base64Bcs::into_inner)
        .inspect(|o| {
            object_keys
                .iter()
                .position(|k| k.address == o.object_id())
                .map(|p| object_keys.swap_remove(p));
        })
        .collect_vec();

    if !object_keys.is_empty() {
        return Err(Error::MissingData(format!("Objects {object_keys:?}")));
    }
    Ok(returned)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() -> color_eyre::Result<()> {
    use cynic::QueryBuilder as _;

    // Variables don't matter, we just need it so taht `Query::build()` compiles
    let vars = Variables { keys: &[] };

    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r"
    query Query($keys: [ObjectKey!]!) {
      multiGetObjects(keys: $keys) {
        address
        objectBcs
      }
    }
    ");
    Ok(())
}
