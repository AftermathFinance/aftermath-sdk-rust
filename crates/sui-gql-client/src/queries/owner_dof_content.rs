use af_sui_types::{Address as SuiAddress, ObjectId};
use cynic::GraphQlResponse;

use super::Error;
use super::fragments::MoveValueRaw;
use super::outputs::{ObjectKey, RawMoveStruct, RawMoveValue};
use crate::{GraphQlClient, GraphQlResponseExt, missing_data, scalars, schema};

pub async fn query<C: GraphQlClient>(
    client: &C,
    address: SuiAddress,
    RawMoveValue {
        type_: df_name_type,
        bcs: df_name_bcs,
    }: RawMoveValue,
    root_version: Option<u64>,
) -> Result<(ObjectKey, RawMoveStruct), Error<C::Error>> {
    let vars = QueryVariables {
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
        .owner
        .ok_or(missing_data!("Owner not found"))?
        .dynamic_object_field
        .ok_or(missing_data!("Dynamic object field not found"))?
        .value
        .ok_or(missing_data!("No dynamic field value"))?;
    match df_value {
        DynamicFieldValue::MoveObject(MoveObject {
            object_id,
            version,
            contents,
        }) => {
            let struct_ = contents
                .ok_or(missing_data!("No contents for DOF"))?
                .try_into()
                .expect("Only Move structs can be top-level objects");
            Ok((ObjectKey { object_id, version }, struct_))
        }
        DynamicFieldValue::Unknown => Err(missing_data!("Not a dynamic object field")),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;
    let vars = QueryVariables {
        address: SuiAddress::new(rand::random()),
        root_version: None,
        name: DynamicFieldName {
            type_: scalars::TypeTag("0x2::sui::SUI".parse().unwrap()),
            bcs: scalars::Base64::new(vec![]),
        },
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r"
    query Query($address: SuiAddress!, $name: DynamicFieldName!, $rootVersion: UInt53) {
      owner(address: $address, rootVersion: $rootVersion) {
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
          }
        }
      }
    }
    ");
}

// ================================================================================
//  Mostly autogenerated by: https://generator.cynic-rs.dev/
// ================================================================================

#[derive(cynic::QueryVariables, Debug)]
struct QueryVariables {
    address: SuiAddress,
    name: DynamicFieldName,
    root_version: Option<af_sui_types::Version>,
}

#[derive(cynic::InputObject, Debug)]
struct DynamicFieldName {
    #[cynic(rename = "type")]
    type_: scalars::TypeTag,
    bcs: scalars::Base64<Vec<u8>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "QueryVariables")]
struct Query {
    #[arguments(address: $address, rootVersion: $root_version)]
    owner: Option<Owner>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "QueryVariables")]
struct Owner {
    #[arguments(name: $name)]
    dynamic_object_field: Option<_DynamicField>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField")]
struct _DynamicField {
    value: Option<DynamicFieldValue>,
}

#[derive(cynic::InlineFragments, Debug)]
enum DynamicFieldValue {
    MoveObject(MoveObject),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
struct MoveObject {
    #[cynic(rename = "address")]
    object_id: ObjectId,
    version: af_sui_types::Version,
    contents: Option<MoveValueRaw>,
}
