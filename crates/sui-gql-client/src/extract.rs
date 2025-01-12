//! Defines [`extract!`](crate::extract!) and its [`Error`].

/// Error for [`extract!`](crate::extract!).
#[derive(thiserror::Error, Debug)]
#[error("Missing data from response: {0}")]
pub struct Error(pub(crate) String);

impl Error {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Helper for extracting data from GraphQL responses.
///
/// Designed specially to deal with the multitude of nested [`Option`]s commonly found in GQL
/// responses. The macro will generate an error string with the full path to the missing attribute.
///
/// # Example
///
/// ```no_run
/// # use af_sui_types::Address;
/// # use sui_gql_schema::{schema, scalars};
/// #
/// # #[derive(cynic::QueryVariables, Debug)]
/// # struct QueryVariables<'a> {
/// #     ch: Address,
/// #     vault: DynamicFieldName<'a>,
/// # }
/// #
/// # #[derive(cynic::InputObject, Debug)]
/// # struct DynamicFieldName<'a> {
/// #     #[cynic(rename = "type")]
/// #     type_: &'a str,
/// #     bcs: scalars::Base64<Vec<u8>>,
/// # }
/// #
/// #[derive(cynic::QueryFragment, Debug)]
/// #[cynic(variables = "QueryVariables")]
/// struct Query {
///     #[arguments(address: $ch)]
///     object: Option<Object>,
/// }
///
/// #[derive(cynic::QueryFragment, Debug)]
/// #[cynic(variables = "QueryVariables")]
/// struct Object {
///     #[arguments(name: $vault)]
///     dynamic_field: Option<DynamicField>,
/// }
///
/// #[derive(cynic::QueryFragment, Debug)]
/// struct DynamicField {
///     value: Option<DynamicFieldValue>,
/// }
///
/// #[derive(cynic::InlineFragments, Debug)]
/// enum DynamicFieldValue {
///     MoveValue(MoveValue),
///     #[cynic(fallback)]
///     Unknown
/// }
///
/// #[derive(cynic::QueryFragment, Debug)]
/// struct MoveValue {
///     #[cynic(rename = "type")]
///     type_: MoveType,
///     bcs: scalars::Base64<Vec<u8>>,
///     __typename: String,
/// }
///
/// #[derive(cynic::QueryFragment, Debug)]
/// struct MoveType {
///     repr: String,
/// }
///
/// use sui_gql_client::extract;
///
/// // Could be obtained in practice from `sui_gql_client::GraphQlResponseExt::try_into_data`
/// let data: Option<Query> = None;
/// let df_value: MoveValue = extract!(
///     data?.object?.dynamic_field?.value?.as_variant(DynamicFieldValue::MoveValue)
/// );
/// # color_eyre::eyre::Ok(())
/// ```
#[macro_export]
macro_rules! extract {
    ($ident:ident $($tt:tt)*) => {{
        $crate::extract!( @attributes [$ident][$($tt)*] -> {
            $ident
        } )
    }};

    (@attributes [$($path:tt)*][] -> { $($expr:tt)* } ) => { $($expr)* };

    (@attributes [$($path:tt)*][.as_variant($($var:tt)+) $($tt:tt)*] -> { $($expr:tt)* } ) => {
        $crate::extract!(@attributes
            [$($path)*.as_variant($($var)*)][$($tt)*] -> {{
                let value = $($expr)*;
                let $($var)+(var) = value else {
                    let msg = stringify!($($path)*.as_variant($($var)*));
                    return Err($crate::extract::Error::new(msg).into());
                };
                var
            }}
        )
    };

    (@attributes [$($path:tt)*][? $($tt:tt)*] -> { $($expr:tt)* } ) => {
        $crate::extract!(@attributes
            [$($path)*][$($tt)*] -> {
                $($expr)*
                .ok_or_else(|| {
                    let msg = stringify!($($path)*);
                    $crate::extract::Error::new(msg)
                })?
            }
        )
    };

    (@attributes [$($path:tt)*][.$ident:ident $($tt:tt)*] -> { $($expr:tt)* } ) => {
        $crate::extract!(@attributes
            [$($path)*.$ident][$($tt)*] -> {
                $($expr)*
                .$ident
            }
        )
    };
}

#[cfg(test)]
mod tests {
    use af_sui_types::Address;
    use sui_gql_schema::{scalars, schema};

    #[derive(cynic::QueryVariables, Debug)]
    #[expect(dead_code, reason = "for cynic")]
    struct Variables<'a> {
        ch: Address,
        vault: DynamicFieldName<'a>,
    }

    #[derive(cynic::InputObject, Debug)]
    struct DynamicFieldName<'a> {
        #[cynic(rename = "type")]
        type_: &'a str,
        bcs: scalars::Base64<Vec<u8>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(variables = "Variables")]
    struct Query {
        #[arguments(address: $ch)]
        object: Option<Object>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(variables = "Variables")]
    struct Object {
        #[arguments(name: $vault)]
        dynamic_field: Option<DynamicField>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    struct DynamicField {
        value: Option<DynamicFieldValue>,
    }

    #[derive(cynic::InlineFragments, Debug)]
    enum DynamicFieldValue {
        MoveValue(MoveValue),
        #[cynic(fallback)]
        Unknown,
    }

    #[derive(cynic::QueryFragment, Debug)]
    struct MoveValue {
        __typename: String,
    }

    #[test]
    fn error_display() {
        fn extract_data(data: Option<Query>) -> Result<MoveValue, super::Error> {
            Ok(extract!(data?
                .object?
                .dynamic_field?
                .value?
                .as_variant(DynamicFieldValue::MoveValue)))
        }

        let data = Some(Query {
            object: Some(Object {
                dynamic_field: Some(DynamicField {
                    value: Some(DynamicFieldValue::Unknown),
                }),
            }),
        });

        let res = extract_data(data);
        assert!(res.is_err());
        insta::assert_snapshot!(res.err().unwrap(), @"Missing data from response: data.object.dynamic_field.value.as_variant(DynamicFieldValue::MoveValue)");

        let res = extract_data(None);
        assert!(res.is_err());
        insta::assert_snapshot!(res.err().unwrap(), @"Missing data from response: data");

        let res = extract_data(Some(Query {
            object: Some(Object {
                dynamic_field: None,
            }),
        }));
        assert!(res.is_err());
        insta::assert_snapshot!(res.err().unwrap(), @"Missing data from response: data.object.dynamic_field");
    }
}
