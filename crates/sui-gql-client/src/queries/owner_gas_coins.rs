use af_move_type::MoveInstance;
use af_move_type::otw::Otw;
use af_sui_types::{Address, ObjectRef};
use futures::Stream;
use sui_framework_sdk::coin::Coin;
use sui_gql_schema::scalars::Base64Bcs;

use super::model::fragments::{ObjectFilter, PageInfo};
use super::stream;
use crate::queries::Error;
use crate::queries::model::fragments::{Checkpoint, ObjectConnection};
use crate::{GraphQlClient, GraphQlResponseExt, schema};

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    filter: ObjectFilter,
    after: Option<String>,
    first: Option<i32>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    checkpoint: Option<Checkpoint>,
    #[arguments(filter: $filter, first: $first, after: $after)]
    objects: Option<ObjectConnection>,
}

pub(super) fn query<C: GraphQlClient>(
    client: &C,
    owner: Address,
    type_: Option<String>,
    page_size: Option<u32>,
) -> impl Stream<Item = super::Result<(u64, ObjectRef, u64), C>> {
    let coin_type = if let Some(_ct) = &type_ {
        type_
    } else {
        Some("0x2::coin::Coin".to_string())
    };

    let filter = ObjectFilter {
        owner: Some(owner),
        type_: coin_type,
        owner_kind: None,
    };
    let vars = Variables {
        after: None,
        first: page_size.map(|v| v.try_into().unwrap_or(i32::MAX)),
        filter,
    };
    stream::forward(client, vars, request)
}

async fn request<C: GraphQlClient>(
    client: &C,
    vars: Variables,
) -> super::Result<
    stream::Page<impl Iterator<Item = super::Result<(u64, ObjectRef, u64), C>> + use<C>>,
    C,
> {
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?;

    let stream::Page { info, data } = extract(data)?;
    Ok(stream::Page::new(
        info,
        data.map(|r| r.map_err(Error::MissingData)),
    ))
}

#[expect(clippy::type_complexity)]
fn extract(
    data: Option<Query>,
) -> Result<stream::Page<impl Iterator<Item = Result<(u64, ObjectRef, u64), String>>>, &'static str>
{
    graphql_extract::extract!(data => {
        checkpoint?
        objects? {
            nodes[] {
                id
                object
            }
            page_info
        }
    });

    let nodes = nodes
        .into_iter()
        .map(move |r: Result<_, &'static str>| -> Result<_, String> {
            let (id, bcs) = r?;
            let obj = bcs
                .ok_or_else(|| format!("BCS for object {id}"))
                .map(Base64Bcs::into_inner)?;

            let coin_obj = obj
                .as_struct()
                .ok_or_else(|| format!("Object {id} is not a struct"))?;
            let coin_obj: MoveInstance<Coin<Otw>> =
                MoveInstance::from_raw_struct(coin_obj.object_type().clone(), coin_obj.contents())
                    .map_err(|e| format!("Error from raw struct {e}"))?;
            let obj_ref = (obj.object_id(), obj.version(), obj.digest());
            Ok((
                checkpoint.sequence_number,
                obj_ref,
                coin_obj.value.balance.value,
            ))
        });
    Ok(stream::Page::new(page_info, nodes))
}

impl stream::UpdatePageInfo for Variables {
    fn update_page_info(&mut self, info: &PageInfo) {
        self.after.clone_from(&info.end_cursor)
    }
}

#[cfg(test)]
#[test]
fn gql_output() -> color_eyre::Result<()> {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        filter: ObjectFilter {
            type_: None,
            owner: None,
            owner_kind: None,
        },
        after: None,
        first: None,
    };

    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r"
    query Query($filter: ObjectFilter!, $after: String, $first: Int) {
      checkpoint {
        sequenceNumber
      }
      objects(filter: $filter, first: $first, after: $after) {
        nodes {
          address
          objectBcs
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
    ");
    Ok(())
}
