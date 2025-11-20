use af_sui_types::Address as SuiAddress;
use cynic::GraphQlResponse;

use super::Error;
use super::model::outputs::RawMoveValue;
use crate::queries::model::fragments::{
    DynamicFieldByName,
    DynamicFieldName,
    DynamicFieldValue,
    MoveObject,
    ObjectKey,
};
use crate::queries::model::outputs::DynamicField as OutputDf;
use crate::{GraphQlClient, GraphQlResponseExt, missing_data, scalars, schema};

#[derive(cynic::QueryVariables, Debug)]
struct Variables {
    address: SuiAddress,
    name: DynamicFieldName,
    at_checkpoint: Option<u64>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $address, atCheckpoint: $at_checkpoint)]
    pub object: Option<ObjectDynamicField>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Object")]
#[cynic(variables = "Variables")]
struct ObjectDynamicField {
    #[arguments(name: $name)]
    dynamic_object_field: Option<DynamicFieldByName>,
}

pub async fn query<C: GraphQlClient>(
    client: &C,
    address: SuiAddress,
    RawMoveValue {
        type_: df_name_type,
        bcs: df_name_bcs,
    }: RawMoveValue,
    at_checkpoint: Option<u64>,
) -> Result<OutputDf, Error<C::Error>> {
    let vars = Variables {
        address,
        at_checkpoint,
        name: DynamicFieldName {
            type_: scalars::TypeTag(df_name_type),
            bcs: scalars::Base64::new(df_name_bcs),
        },
    };
    let result: GraphQlResponse<Query> = client.query(vars).await.map_err(Error::Client)?;
    let data = result
        .try_into_data()?
        .ok_or(missing_data!("Response empty"))?;
    let df_value = data
        .object
        .ok_or(missing_data!("Object not found"))?
        .dynamic_object_field
        .ok_or(missing_data!("Dynamic object field not found"))?
        .value
        .ok_or(missing_data!("No dynamic object field value"))?;

    let out = match df_value {
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

    Ok(out)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;
    let vars = Variables {
        address: SuiAddress::new(rand::random()),
        at_checkpoint: None,
        name: DynamicFieldName {
            type_: scalars::TypeTag("0x2::sui::SUI".parse().unwrap()),
            bcs: scalars::Base64::new(vec![]),
        },
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r"
    query Query($address: SuiAddress!, $name: DynamicFieldName!, $atCheckpoint: UInt53) {
      object(address: $address, atCheckpoint: $atCheckpoint) {
        dynamicObjectField(name: $name) {
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
      }
    }
    ");
}
