use af_sui_types::{ObjectId, Version};

use super::fragments::{ObjectFilter, PageInfoForward};
use super::Error;
use crate::{missing_data, schema, GraphQlClient, Paged, PagedResponse};

pub async fn query<C: GraphQlClient>(
    client: &C,
    package_ids: Vec<ObjectId>,
) -> Result<impl Iterator<Item = (ObjectId, u64, u64)>, Error<C::Error>> {
    #[expect(
        deprecated,
        reason = "TODO: build query from scratch with new ObjectFilter"
    )]
    let vars = QueryVariables {
        filter: Some(ObjectFilter {
            type_: None,
            owner: None,
            object_ids: Some(package_ids),
            object_keys: None,
        }),
        first: None,
        after: None,
    };

    let response: PagedResponse<Query> = client.query_paged(vars).await.map_err(Error::Client)?;
    let (init, pages) = response
        .try_into_data()?
        .ok_or_else(|| missing_data!("No data"))?;

    Ok(init
        .objects
        .nodes
        .into_iter()
        .chain(pages.into_iter().flat_map(|p| p.objects.nodes))
        .filter_map(|o| {
            let effects = o.as_move_package?.previous_transaction_block?.effects?;
            Some((
                o.address,
                effects.epoch?.epoch_id,
                effects.checkpoint?.sequence_number,
            ))
        }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = QueryVariables {
        filter: None,
        first: None,
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query);
}

// ================================================================================

impl Paged for Query {
    type Input = QueryVariables;
    type NextPage = Self;
    type NextInput = QueryVariables;

    fn next_variables(&self, mut prev_vars: Self::Input) -> Option<Self::NextInput> {
        let PageInfoForward {
            has_next_page,
            end_cursor,
        } = &self.objects.page_info;
        if *has_next_page {
            prev_vars.after.clone_from(end_cursor);
            Some(prev_vars)
        } else {
            None
        }
    }
}

// ================================================================================

#[derive(cynic::QueryVariables, Clone, Debug)]
struct QueryVariables {
    after: Option<String>,
    filter: Option<ObjectFilter>,
    first: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "QueryVariables")]
struct Query {
    #[arguments(filter: $filter, first: $first, after: $after)]
    objects: ObjectConnection,
}

#[derive(cynic::QueryFragment, Debug)]
struct ObjectConnection {
    nodes: Vec<Object>,
    page_info: PageInfoForward,
}

#[derive(cynic::QueryFragment, Debug)]
struct Object {
    address: ObjectId,
    as_move_package: Option<MovePackage>,
}

#[derive(cynic::QueryFragment, Debug)]
struct MovePackage {
    previous_transaction_block: Option<TransactionBlock>,
}

#[derive(cynic::QueryFragment, Debug)]
struct TransactionBlock {
    effects: Option<TransactionBlockEffects>,
}

#[derive(cynic::QueryFragment, Debug)]
struct TransactionBlockEffects {
    epoch: Option<Epoch>,
    checkpoint: Option<Checkpoint>,
}

#[derive(cynic::QueryFragment, Debug)]
struct Epoch {
    epoch_id: Version,
}

#[derive(cynic::QueryFragment, Debug)]
struct Checkpoint {
    sequence_number: Version,
}
