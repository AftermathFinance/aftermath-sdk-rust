use std::str::FromStr;

use af_sui_types::{encoding, Address as SuiAddress, ObjectId};
use cynic::impl_scalar;
use derive_more::{AsRef, Deref, Display, From, Into};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_with::{base64, serde_as, Bytes, DisplayFromStr};

use crate::schema;

macro_rules! scalar_with_generics {
    (
        impl<$($T:ident),+> $schema:ident::$scalar:ident for $type_:ty $(where { $($bounds:tt)+ })?
    ) => {
        impl<$($T),+> cynic::schema::IsScalar<$schema::$scalar> for $type_
        $(where $($bounds)+)?
        {
            type SchemaType = $schema::$scalar;
        }

        impl<$($T),+> cynic::coercions::CoercesTo<$schema::$scalar> for $type_
        $(where $($bounds)+)?
        {
        }

        impl<$($T),+> $schema::variable::Variable for $type_
        $(where $($bounds)+)?
        {
            const TYPE: cynic::variables::VariableType = cynic::variables::VariableType::Named(
                <$schema::$scalar as cynic::schema::NamedType>::NAME,
            );
        }
    };
}

// =============================================================================
//  Base64
// =============================================================================

/// Base64-encoded data. Received from the server as a string.
///
/// From the schema: "String containing Base64-encoded binary data."
#[serde_as]
#[derive(AsRef, Clone, Deref, Deserialize, Serialize)]
#[as_ref(forward)]
pub struct Base64<T>(#[serde_as(as = "base64::Base64")] T)
where
    T: AsRef<[u8]> + From<Vec<u8>>;

impl<T> Base64<T>
where
    T: AsRef<[u8]> + From<Vec<u8>>,
{
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::fmt::Debug for Base64<T>
where
    T: AsRef<[u8]> + From<Vec<u8>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Base64({})",
            af_sui_types::encode_base64_default(&self.0)
        )
    }
}

scalar_with_generics! {
    impl<T> schema::Base64 for Base64<T> where {
        T: AsRef<[u8]> + From<Vec<u8>>,
    }
}

#[serde_as]
#[derive(AsRef, Clone, Debug, Deref, Deserialize, Serialize)]
#[as_ref(forward)]
#[serde(bound(deserialize = "T: for<'a> Deserialize<'a>"))]
#[serde(bound(serialize = "T: Serialize"))]
pub struct Base64Bcs<T>(#[serde_as(as = "encoding::Base64Bcs")] T);

impl<T> Base64Bcs<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

scalar_with_generics! {
    impl<T> schema::Base64 for Base64Bcs<T>
}

// =============================================================================
//  BigInt
// =============================================================================

/// Generic integer. Received from the server as a string.
///
/// From the schema: "String representation of an arbitrary width, possibly signed integer."
#[serde_as]
#[derive(Clone, Debug, Display, Deserialize, Serialize)]
pub struct BigInt<T>(#[serde_as(as = "DisplayFromStr")] T)
where
    T: Display + FromStr,
    T::Err: Display;

impl<T> BigInt<T>
where
    T: Display + FromStr,
    T::Err: Display,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

scalar_with_generics! {
    impl<T> schema::BigInt for BigInt<T>
    where {
        T: Display + FromStr,
        T::Err: Display,
    }
}

// =============================================================================
//  DateTime
// =============================================================================

impl_scalar!(DateTime, schema::DateTime);

/// ISO-8601 Date and Time: RFC3339 in UTC with format: YYYY-MM-DDTHH:MM:SS.mmmZ. Note that the
/// milliseconds part is optional, and it may be omitted if its value is 0.
#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq, Into, Display, Deref)]
pub struct DateTime(#[serde_as(as = "DisplayFromStr")] chrono::DateTime<chrono::Utc>);

// =============================================================================
//  JSON
// =============================================================================

impl_scalar!(Json, schema::JSON);

// =============================================================================
//  MoveData
// =============================================================================

impl_scalar!(MoveData, schema::MoveData);

/// The contents of a Move Value, corresponding to the following recursive type:
///
/// type MoveData =
///     { Address: SuiAddress }
///   | { UID:     SuiAddress }
///   | { ID:      SuiAddress }
///   | { Bool:    bool }
///   | { Number:  BigInt }
///   | { String:  string }
///   | { Vector:  [MoveData] }
///   | { Option:   MoveData? }
///   | { Struct:  [{ name: string, value: MoveData }] }
///   | { Variant: {
///       name: string,
///       fields: [{ name: string, value: MoveData }],
///   }
#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum MoveData {
    Address(#[serde_as(as = "Bytes")] [u8; 32]),
    #[serde(rename = "UID")]
    Uid(#[serde_as(as = "Bytes")] [u8; 32]),
    #[serde(rename = "ID")]
    Id(#[serde_as(as = "Bytes")] [u8; 32]),
    Bool(bool),
    Number(String),
    String(String),
    Vector(Vec<MoveData>),
    Option(Option<Box<MoveData>>),
    Struct(Vec<MoveField>),
    Variant(MoveVariant),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MoveVariant {
    name: String,
    fields: Vec<MoveField>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MoveField {
    pub name: String,
    pub value: MoveData,
}

// =============================================================================
//  MoveTypeLayout
// =============================================================================

impl_scalar!(MoveTypeLayout, schema::MoveTypeLayout);

#[doc = r#"The shape of a concrete Move Type (a type with all its type parameters instantiated with
concrete types), corresponding to the following recursive type:

type MoveTypeLayout =
    "address"
  | "bool"
  | "u8" | "u16" | ... | "u256"
  | { vector: MoveTypeLayout }
  | {
      struct: {
        type: string,
        fields: [{ name: string, layout: MoveTypeLayout }],
      }
    }
  | { enum: [{
          type: string,
          variants: [{ 
              name: string,
              fields: [{ name: string, layout: MoveTypeLayout }],
          }]
      }] 
    }"#]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MoveTypeLayout {
    Address,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Vector(Box<MoveTypeLayout>),
    Struct(MoveStructLayout),
    Enum(MoveEnumLayout),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MoveEnumLayout {
    pub variants: Vec<MoveVariantLayout>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MoveVariantLayout {
    pub name: String,
    pub layout: Vec<MoveFieldLayout>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MoveStructLayout {
    #[serde(rename = "type")]
    type_: String,
    fields: Vec<MoveFieldLayout>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MoveFieldLayout {
    name: String,
    layout: MoveTypeLayout,
}

// =============================================================================
//  MoveTypeSignature
// =============================================================================

impl_scalar!(MoveTypeSignature, schema::MoveTypeSignature);

#[doc = r#"The signature of a concrete Move Type (a type with all its type parameters instantiated
with concrete types, that contains no references), corresponding to the following recursive type:

type MoveTypeSignature =
    "address"
  | "bool"
  | "u8" | "u16" | ... | "u256"
  | { vector: MoveTypeSignature }
  | {
      datatype: {
        package: string,
        module: string,
        type: string,
        typeParameters: [MoveTypeSignature],
      }
    }"#]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MoveTypeSignature {
    Address,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Vector(Box<MoveTypeSignature>),
    Datatype {
        package: String,
        module: String,
        #[serde(rename = "type")]
        type_: String,
        #[serde(rename = "typeParameters")]
        type_parameters: Vec<MoveTypeSignature>,
    },
}

// =============================================================================
//  OpenMoveTypeSignature
// =============================================================================

impl_scalar!(OpenMoveTypeSignature, schema::OpenMoveTypeSignature);

#[doc = r#"The shape of an abstract Move Type (a type that can contain free type parameters, and can
optionally be taken by reference), corresponding to the following recursive type:

type OpenMoveTypeSignature = {
  ref: ("&" | "&mut")?,
  body: OpenMoveTypeSignatureBody,
}

type OpenMoveTypeSignatureBody =
    "address"
  | "bool"
  | "u8" | "u16" | ... | "u256"
  | { vector: OpenMoveTypeSignatureBody }
  | {
      datatype {
        package: string,
        module: string,
        type: string,
        typeParameters: [OpenMoveTypeSignatureBody]
      }
    }
  | { typeParameter: number }"#]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenMoveTypeSignature {
    #[serde(rename = "ref")]
    ref_: Option<OpenMoveTypeReference>,
    body: OpenMoveTypeSignatureBody,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OpenMoveTypeReference {
    #[serde(rename = "&")]
    Immutable,

    #[serde(rename = "&mut")]
    Mutable,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OpenMoveTypeSignatureBody {
    TypeParameter(u16),
    Address,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Vector(Box<OpenMoveTypeSignatureBody>),
    Datatype {
        package: String,
        module: String,
        #[serde(rename = "type")]
        type_: String,
        #[serde(rename = "typeParameters")]
        type_parameters: Vec<OpenMoveTypeSignatureBody>,
    },
}

// =============================================================================
//  SuiAddress:
//  String containing 32B hex-encoded address, with a leading "0x". Leading
//  zeroes can be omitted on input but will always appear in outputs (SuiAddress
//  in output is guaranteed to be 66 characters long).
// =============================================================================

impl_scalar!(ObjectId, schema::SuiAddress);
impl_scalar!(SuiAddress, schema::SuiAddress);

// =============================================================================
//  Extras
// =============================================================================

impl_scalar!(Digest, schema::String);
impl_scalar!(TypeTag, schema::String);

#[derive(Clone, Debug, Deserialize)]
pub struct Digest(pub af_sui_types::Digest);

/// Newtype for using [`af_sui_types::TypeTag`] in GQL queries.
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TypeTag(#[serde_as(as = "DisplayFromStr")] pub af_sui_types::TypeTag);

// =============================================================================
//  UInt53
// =============================================================================

impl_scalar!(af_sui_types::Version, schema::UInt53);

// =============================================================================
//  Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use color_eyre::Result;

    use super::*;

    /// Taken from
    ///
    /// ```grapql
    /// query Events($first: Int, $after: String, $filter: EventFilter) {
    ///   events(after: $after, first: $first, filter: $filter) {
    ///     edges {
    ///       node {
    ///         timestamp
    ///         type {
    ///           signature
    ///         }
    ///         json
    ///       }
    ///       cursor
    ///     }
    ///     pageInfo {
    ///       hasNextPage
    ///     }
    ///   }
    /// }
    /// ```
    /// Variables:
    /// ```json
    /// {
    ///   "filter": {
    ///     "eventType": "0xfd6f306bb2f8dce24dd3d4a9bdc51a46e7c932b15007d73ac0cfb38c15de0fea::events"
    ///   }
    /// }
    /// ```
    const MOVE_TYPE_SIGNATURE_JSON: &str = r#"{
        "datatype": {
          "package": "0xfd6f306bb2f8dce24dd3d4a9bdc51a46e7c932b15007d73ac0cfb38c15de0fea",
          "module": "events",
          "type": "DepositedCollateral",
          "typeParameters": []
        }
    }"#;

    #[test]
    fn move_type_signature_serde() -> Result<()> {
        let sig: MoveTypeSignature = serde_json::from_str(MOVE_TYPE_SIGNATURE_JSON)?;
        assert_eq!(
            sig,
            MoveTypeSignature::Datatype {
                package: "0xfd6f306bb2f8dce24dd3d4a9bdc51a46e7c932b15007d73ac0cfb38c15de0fea"
                    .into(),
                module: "events".into(),
                type_: "DepositedCollateral".into(),
                type_parameters: vec![],
            }
        );
        Ok(())
    }
}
