use af_sui_types::Address as SuiAddress;
use futures_core::Stream;

use super::model::fragments;
use super::model::outputs::{DynamicField as OutputDf, RawMoveValue};
use super::{Error, stream};
use crate::queries::model::fragments::{
    Address,
    DynamicField,
    DynamicFieldConnection,
    DynamicFieldValue,
    MoveObject,
    ObjectKey,
};
use crate::{GraphQlClient, GraphQlResponseExt as _, missing_data, schema};

#[derive(cynic::QueryVariables, Debug, Clone)]
struct Variables {
    address: SuiAddress,
    root_version: Option<af_sui_types::Version>,
    after: Option<String>,
    first: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $address)]
    pub address: Address,
}

pub async fn query<C: GraphQlClient>(
    client: &C,
    address: SuiAddress,
    root_version: Option<u64>,
    page_size: Option<i32>,
) -> impl Stream<Item = super::Result<(RawMoveValue, OutputDf), C>> + '_ {
    let vars = Variables {
        address,
        root_version,
        first: page_size,
        after: None,
    };

    stream::forward(client, vars, request)
}

async fn request<C: GraphQlClient>(
    client: &C,
    vars: Variables,
) -> super::Result<
    stream::Page<
        impl Iterator<Item = super::Result<(RawMoveValue, OutputDf), C>> + 'static + use<C>,
    >,
    C,
> {
    let data = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(missing_data!("Response empty"))?;

    let DynamicFieldConnection { nodes, page_info } = data
        .address
        .dynamic_fields
        .ok_or(missing_data!("No dynamic fields found"))?;

    let data = nodes.into_iter().map(|DynamicField { name, value }| {
        let name = name
            .ok_or(missing_data!("Dynamic field found but with no name"))?
            .try_into()
            .map_err(|e| missing_data!("Dynamic field name content empty. Error: {e}"))?;
        let instance = value.ok_or(missing_data!("Dynamic field found but with no value"))?;
        let out = match instance {
            DynamicFieldValue::MoveObject(MoveObject {
                address,
                version,
                contents,
            }) => {
                let struct_ = contents
                    .ok_or(missing_data!("No contents for DF"))?
                    .try_into()
                    .expect("Only Move structs can be top-level objects");
                OutputDf::Object(
                    ObjectKey {
                        version,
                        address,
                        root_version: None,
                        at_checkpoint: None,
                    },
                    struct_,
                )
            }
            DynamicFieldValue::MoveValue(value) => OutputDf::Field(
                value
                    .try_into()
                    .map_err(|e| missing_data!("Dynamic field name content empty. Error: {e}"))?,
            ),
            DynamicFieldValue::Unknown => return Err(missing_data!("Unknown dynamic field type")),
        };
        Ok((name, out))
    });

    Ok(stream::Page {
        info: page_info,
        data,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;
    let vars = Variables {
        address: SuiAddress::new(rand::random()),
        root_version: None,
        first: None,
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r"
    query Query($address: SuiAddress!) {
      address(address: $address) {
        dynamicFields {
          nodes {
            name {
              type {
                repr
              }
              bcs
            }
            value {
              __typename
              ... on MoveObject {
                address
                version
                contents {
                  type {
                    repr
                  }
                  bcs
                }
              }
              ... on MoveValue {
                type {
                  repr
                }
                bcs
              }
            }
          }
          pageInfo {
            hasNextPage
            endCursor
            hasPreviousPage
            startCursor
          }
        }
      }
    }
    ");
}

impl stream::UpdatePageInfo for Variables {
    fn update_page_info(&mut self, info: &fragments::PageInfo) {
        self.after.clone_from(&info.end_cursor)
    }
}
