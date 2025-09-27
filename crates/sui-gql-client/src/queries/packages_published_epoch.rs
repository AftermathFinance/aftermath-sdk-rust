use af_sui_types::{ObjectId, Version};
use futures::TryStreamExt as _;

use super::fragments::{ObjectFilterV2, PageInfo, PageInfoForward};
use super::{Error, stream};
use crate::{GraphQlClient, GraphQlResponseExt as _, schema};

type Item = (ObjectId, u64, u64);

pub async fn query<C: GraphQlClient>(
    client: &C,
    package_ids: Vec<ObjectId>,
) -> Result<impl Iterator<Item = Item> + use<C>, Error<C::Error>> {
    let vars = QueryVariables {
        filter: Some(ObjectFilterV2 {
            type_: None,
            owner: None,
            object_ids: Some(&package_ids),
        }),
        first: None,
        after: None,
    };

    let results: Vec<_> = stream::forward(client, vars, request).try_collect().await?;

    Ok(results.into_iter())
}

async fn request<C: GraphQlClient>(
    client: &C,
    vars: QueryVariables<'_>,
) -> super::Result<stream::Page<impl Iterator<Item = super::Result<Item, C>> + use<C>>, C> {
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;
    graphql_extract::extract!(data => {
        objects {
            page_info
            nodes[] {
                address
                as_move_package? {
                    previous_transaction_block? {
                        effects? {
                            epoch? {
                                epoch_id
                            }
                            checkpoint? {
                                sequence_number
                            }
                        }
                    }
                }
            }
        }
    });
    Ok(stream::Page::new(
        page_info,
        nodes.map(|r| -> super::Result<_, C> {
            let (address, (epoch_id, ckpt_seq)) = r?;
            Ok((address, epoch_id, ckpt_seq))
        }),
    ))
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
    insta::assert_snapshot!(operation.query, @r###"
    query Query($after: String, $filter: ObjectFilter, $first: Int) {
      objects(filter: $filter, first: $first, after: $after) {
        nodes {
          address
          asMovePackage {
            previousTransactionBlock {
              effects {
                epoch {
                  epochId
                }
                checkpoint {
                  sequenceNumber
                }
              }
            }
          }
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
    "###);
}

// ================================================================================

impl stream::UpdatePageInfo for QueryVariables<'_> {
    fn update_page_info(&mut self, info: &PageInfo) {
        self.after.clone_from(&info.end_cursor)
    }
}

// ================================================================================

#[derive(cynic::QueryVariables, Clone, Debug)]
struct QueryVariables<'a> {
    after: Option<String>,
    filter: Option<ObjectFilterV2<'a>>,
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
