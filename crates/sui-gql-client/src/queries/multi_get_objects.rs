use af_sui_types::{Object, ObjectId, Version};
use itertools::Itertools as _;
use sui_gql_schema::scalars::Base64Bcs;

use super::fragments::ObjectKey;
use crate::queries::Error;
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    objects: impl IntoIterator<Item = (ObjectId, Version)> + Send,
) -> super::Result<Vec<Object>, C> {
    let mut object_keys = objects
        .into_iter()
        .sorted()
        .dedup()
        .map(|(object_id, version)| ObjectKey { object_id, version })
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
                .position(|k| k.object_id == o.object_id() && k.version == o.version())
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
    insta::assert_snapshot!(operation.query, @r###"
    query Query($keys: [ObjectKey!]!) {
      multiGetObjects(keys: $keys) {
        object: bcs
      }
    }
    "###);
    Ok(())
}

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

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Object")]
struct ObjectGql {
    #[cynic(alias, rename = "bcs")]
    object: Option<Base64Bcs<Object>>,
}
