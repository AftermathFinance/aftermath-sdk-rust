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
    root_version: Option<af_sui_types::Version>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $address)]
    pub address: AddressDynamicField,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Address")]
#[cynic(variables = "Variables")]
struct AddressDynamicField {
    #[arguments(name: $name)]
    dynamic_field: Option<DynamicFieldByName>,
}

pub async fn query<C: GraphQlClient>(
    client: &C,
    address: SuiAddress,
    RawMoveValue {
        type_: df_name_type,
        bcs: df_name_bcs,
    }: RawMoveValue,
    root_version: Option<u64>,
) -> Result<OutputDf, Error<C::Error>> {
    let vars = Variables {
        address,
        root_version,
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
        .address
        .dynamic_field
        .ok_or(missing_data!("Dynamic field not found"))?
        .value
        .ok_or(missing_data!("No dynamic field value"))?;

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
        root_version: None,
        name: DynamicFieldName {
            type_: scalars::TypeTag("0x2::sui::SUI".parse().unwrap()),
            bcs: scalars::Base64::new(vec![]),
        },
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r"
    query Query($address: SuiAddress!, $name: DynamicFieldName!) {
      address(address: $address) {
        dynamicField(name: $name) {
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
