#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Defines the core standard for representing Move types off-chain and their type tags.
//!
//! The core items are [`MoveType`](crate::MoveType) and [`MoveTypeTag`](crate::MoveTypeTag). These
//! are useful trait bounds to use when dealing with generic off-chain Move type representations.
//! They are implemented for the primitive types that correspond to Move's primitives
//! (integers/bool). Also included is [`MoveVec`](crate::vector::MoveVec), corresponding to `vector`
//! and defining a pretty [`Display`](::std::fmt::Display).
//!
//! For Move structs (objects), [`MoveStruct`](crate::MoveStruct) should be used as it has an
//! associated [`MoveStructTag`](crate::MoveStructTag). The
//! [`MoveStruct`](af_move_type_derive::MoveStruct) derive macro is exported for automatically
//! creating a `MoveStructTag` implementation from normal Rust struct declarations.
//!
//! A specific instance of a Move type is represented by [`MoveInstance`](crate::MoveInstance).
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

pub use af_move_type_derive::MoveStruct;
use af_sui_types::u256::U256;
use af_sui_types::{Address, IdentStr, Identifier, ObjectId, StructTag, TypeTag};
use serde::{Deserialize, Serialize};

#[cfg(feature = "bson")]
#[deprecated = "Unmaintained"]
pub mod bson_serde;
#[doc(hidden)]
pub mod external;
pub mod otw;
pub mod vector;

// =============================================================================
//  Errors
// =============================================================================

#[derive(thiserror::Error, Debug)]
pub enum TypeTagError {
    #[error("Wrong TypeTag variant: expected {expected}, got {got}")]
    Variant { expected: String, got: TypeTag },
    #[error("StructTag params: {0}")]
    StructTag(#[from] StructTagError),
}

#[derive(thiserror::Error, Debug)]
pub enum StructTagError {
    #[error("Wrong address: expected {expected}, got {got}")]
    Address { expected: Address, got: Address },
    #[error("Wrong module: expected {expected}, got {got}")]
    Module {
        expected: Identifier,
        got: Identifier,
    },
    #[error("Wrong name: expected {expected}, got {got}")]
    Name {
        expected: Identifier,
        got: Identifier,
    },
    #[error("Wrong type parameters: {0}")]
    TypeParams(#[from] TypeParamsError),
}

#[derive(thiserror::Error, Debug)]
pub enum TypeParamsError {
    #[error("Wrong number of generics: expected {expected}, got {got}")]
    Number { expected: usize, got: usize },
    #[error("Wrong type for generic: {0}")]
    TypeTag(Box<TypeTagError>),
}

impl From<TypeTagError> for TypeParamsError {
    fn from(value: TypeTagError) -> Self {
        Self::TypeTag(Box::new(value))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseTypeTagError {
    #[error("Parsing TypeTag: {0}")]
    FromStr(#[from] sui_sdk_types::types::TypeParseError),
    #[error("Converting from TypeTag: {0}")]
    TypeTag(#[from] TypeTagError),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseStructTagError {
    #[error("Parsing StructTag: {0}")]
    FromStr(#[from] sui_sdk_types::types::TypeParseError),
    #[error("Converting from StructTag: {0}")]
    StructTag(#[from] StructTagError),
}

#[derive(thiserror::Error, Debug)]
pub enum FromRawTypeError {
    #[error("Converting from TypeTag: {0}")]
    TypeTag(#[from] TypeTagError),
    #[error("Deserializing BCS: {0}")]
    Bcs(#[from] bcs::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum FromRawStructError {
    #[error("Converting from StructTag: {0}")]
    StructTag(#[from] StructTagError),
    #[error("Deserializing BCS: {0}")]
    Bcs(#[from] bcs::Error),
}

// =============================================================================
//  MoveType
// =============================================================================

/// Trait marking a Move data type. Has a specific way to construct a `TypeTag`.
pub trait MoveType:
    Clone
    + std::fmt::Debug
    + std::fmt::Display
    + for<'de> Deserialize<'de>
    + Serialize
    + PartialEq
    + Eq
    + std::hash::Hash
{
    type TypeTag: MoveTypeTag;

    /// Deserialize the contents of the Move type from BCS bytes.
    fn from_bcs(bytes: &[u8]) -> bcs::Result<Self> {
        bcs::from_bytes(bytes)
    }

    /// Consuming version of [`to_bcs`](MoveType::to_bcs).
    fn into_bcs(self) -> bcs::Result<Vec<u8>> {
        bcs::to_bytes(&self)
    }

    /// Serialize the contents of the Move type to BCS bytes.
    fn to_bcs(&self) -> bcs::Result<Vec<u8>> {
        bcs::to_bytes(self)
    }

    #[cfg(feature = "bson")]
    /// Serialize the contents of the Move type to BSON.
    #[expect(deprecated)]
    fn to_bson(&self) -> bson_serde::ser::Result<bson::Bson> {
        bson_serde::to_move_bson(self)
    }

    /// Consuming version of [`to_json`](MoveType::to_json).
    fn into_json(self) -> serde_json::Value {
        let mut value = serde_json::json!(self);
        // Move only uses integer values, for which the JSON encoding uses strings
        number_to_string_value_recursive(&mut value);
        value
    }

    /// Serialize the contents of the Move type to JSON.
    ///
    /// The method takes care to use JSON [`String`](serde_json::Value::String) representations for
    /// integer types, for which [`serde`] would use [`Number`](serde_json::Value::Number).
    ///
    /// This is useful for interacting with the RPC.
    fn to_json(&self) -> serde_json::Value {
        let mut value = serde_json::json!(self);
        // Move only uses integer values, for which the JSON encoding uses strings
        number_to_string_value_recursive(&mut value);
        value
    }
}

pub trait MoveTypeTag:
    Into<TypeTag>
    + TryFrom<TypeTag, Error = TypeTagError>
    + FromStr
    + Clone
    + Debug
    + PartialEq
    + Eq
    + Hash
    + for<'de> Deserialize<'de>
    + PartialOrd
    + Ord
    + Serialize
{
}

impl<T> MoveTypeTag for T where
    T: Into<TypeTag>
        + TryFrom<TypeTag, Error = TypeTagError>
        + FromStr
        + Clone
        + Debug
        + PartialEq
        + Eq
        + Hash
        + for<'de> Deserialize<'de>
        + PartialOrd
        + Ord
        + Serialize
{
}

// =============================================================================
//  MoveStruct
// =============================================================================

/// Trait marking a Move struct type. Has a specific way to construct a `StructTag`.
pub trait MoveStruct: MoveType<TypeTag = Self::StructTag> {
    type StructTag: MoveStructTag;
}

pub trait MoveStructTag:
    Into<StructTag> + TryFrom<StructTag, Error = StructTagError> + MoveTypeTag
{
}

impl<T> MoveStructTag for T where
    T: Into<StructTag> + TryFrom<StructTag, Error = StructTagError> + MoveTypeTag
{
}

// =============================================================================
//  Abilities
// =============================================================================

pub trait HasKey: MoveStruct {
    fn object_id(&self) -> ObjectId;
}

pub trait HasCopy: MoveStruct + Copy {}

pub trait HasStore: MoveStruct {}

pub trait HasDrop: MoveStruct {}

// =============================================================================
//  Static attributes
// =============================================================================

/// Move type for which the type tag can be derived at compile time.
pub trait StaticTypeTag: MoveType {
    fn type_() -> Self::TypeTag;

    fn type_tag() -> TypeTag {
        Self::type_().into()
    }
}

/// Move struct for which the address of the package is known at compile time.
pub trait StaticAddress: MoveStruct {
    fn address() -> Address;
}

/// Move struct for which the module in the package is known at compile time.
pub trait StaticModule: MoveStruct {
    fn module() -> Identifier;
}

/// Move struct for which the name of object is known at compile time.
pub trait StaticName: MoveStruct {
    fn name() -> Identifier;
}

/// Move struct for which the type args of object are known at compile time.
pub trait StaticTypeParams: MoveStruct {
    fn type_params() -> Vec<TypeTag>;
}

/// Move struct for which the struct tag can be derived at compile time.
pub trait StaticStructTag: MoveStruct {
    fn struct_tag() -> StructTag;
}

impl<T> StaticStructTag for T
where
    T: StaticAddress + StaticModule + StaticName + StaticTypeParams,
{
    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::address(),
            module: Self::module(),
            name: Self::name(),
            type_params: Self::type_params(),
        }
    }
}

// =============================================================================
//  MoveInstance
// =============================================================================

/// Represents an instance of a Move type.
///
/// Both `type_` and `value` are necessary to represent an instance since otherwise there would be
/// ambiguity, e.g., when the same package is published twice on-chain.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MoveInstance<T: MoveType> {
    pub type_: T::TypeTag,
    pub value: T,
}

impl<T: StaticTypeTag> From<T> for MoveInstance<T> {
    fn from(value: T) -> Self {
        Self {
            type_: T::type_(),
            value,
        }
    }
}

impl<T: MoveStruct + tabled::Tabled> std::fmt::Display for MoveInstance<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use tabled::settings::panel::Header;
        use tabled::settings::{Rotate, Settings, Style};
        use tabled::Table;

        let stag: StructTag = self.type_.clone().into();
        let settings = Settings::default()
            .with(Rotate::Left)
            .with(Rotate::Top)
            .with(Style::rounded())
            .with(Header::new(stag.to_string()));
        let mut table = Table::new([&self.value]);
        table.with(settings);
        write!(f, "{table}")
    }
}

impl std::fmt::Display for MoveInstance<Address> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

macro_rules! impl_primitive_move_instance_display {
    ($($type:ty)+) => {$(
        impl std::fmt::Display for MoveInstance<$type> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.value)
            }
        }
    )+};
}

impl_primitive_move_instance_display! {
    bool
    u8
    u16
    u32
    u64
    u128
    U256
}

impl<T: MoveType> MoveInstance<T> {
    /// Convenience function for constructing from raw RPC-returned general types.
    pub fn from_raw_type(tag: TypeTag, bytes: &[u8]) -> Result<Self, FromRawTypeError> {
        Ok(Self {
            type_: tag.try_into()?,
            value: T::from_bcs(bytes)?,
        })
    }
}

impl<T: MoveStruct> MoveInstance<T> {
    /// Convenience function for constructing from raw RPC-returned structs.
    pub fn from_raw_struct(stag: StructTag, bytes: &[u8]) -> Result<Self, FromRawStructError> {
        Ok(Self {
            type_: stag.try_into()?,
            value: T::from_bcs(bytes)?,
        })
    }
}

fn number_to_string_value_recursive(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(a) => {
            for v in a {
                number_to_string_value_recursive(v)
            }
        }
        serde_json::Value::Number(n) => *value = serde_json::Value::String(n.to_string()),
        serde_json::Value::Object(o) => {
            for v in o.values_mut() {
                number_to_string_value_recursive(v)
            }
        }
        _ => (),
    }
}

// =============================================================================
// Trait impls
// =============================================================================

macro_rules! impl_primitive_type_tags {
    ($($typ:ty: ($type_:ident, $variant:ident)),*) => {
        $(
            #[derive(
                Clone,
                Debug,
                PartialEq,
                Eq,
                Hash,
                Deserialize,
                PartialOrd,
                Ord,
                Serialize
            )]
            pub struct $type_;

            impl From<$type_> for TypeTag {
                fn from(_value: $type_) -> Self {
                    Self::$variant
                }
            }

            impl TryFrom<TypeTag> for $type_ {
                type Error = TypeTagError;

                fn try_from(value: TypeTag) -> Result<Self, Self::Error> {
                    match value {
                        TypeTag::$variant => Ok(Self),
                        _ => Err(TypeTagError::Variant {
                            expected: stringify!($variant).to_owned(),
                            got: value }
                        )
                    }
                }
            }

            impl FromStr for $type_ {
                type Err = ParseTypeTagError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let tag: TypeTag = s.parse()?;
                    Ok(tag.try_into()?)
                }
            }

            impl MoveType for $typ {
                type TypeTag = $type_;
            }

            impl StaticTypeTag for $typ {
                fn type_() -> Self::TypeTag {
                    $type_ {}
                }
            }
        )*
    };
}

impl_primitive_type_tags! {
    Address: (AddressTypeTag, Address),
    bool: (BoolTypeTag, Bool),
    u8: (U8TypeTag, U8),
    u16: (U16TypeTag, U16),
    u32: (U32TypeTag, U32),
    u64: (U64TypeTag, U64),
    u128: (U128TypeTag, U128),
    U256: (U256TypeTag, U256)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, PartialOrd, Ord, Serialize)]
pub struct StringTypeTag;

impl From<StringTypeTag> for TypeTag {
    fn from(value: StringTypeTag) -> Self {
        Self::Struct(Box::new(value.into()))
    }
}

impl TryFrom<TypeTag> for StringTypeTag {
    type Error = TypeTagError;

    fn try_from(value: TypeTag) -> Result<Self, Self::Error> {
        match value {
            TypeTag::Struct(stag) => Ok((*stag).try_into()?),
            other => Err(TypeTagError::Variant {
                expected: "Struct(_)".to_owned(),
                got: other,
            }),
        }
    }
}

impl From<StringTypeTag> for StructTag {
    fn from(_: StringTypeTag) -> Self {
        String::struct_tag()
    }
}

impl TryFrom<StructTag> for StringTypeTag {
    type Error = StructTagError;

    fn try_from(value: StructTag) -> Result<Self, Self::Error> {
        use StructTagError::*;
        let StructTag {
            address,
            module,
            name,
            type_params,
        } = value;
        let expected = String::struct_tag();
        if address != expected.address {
            return Err(Address {
                expected: expected.address,
                got: address,
            });
        }
        if module != expected.module {
            return Err(Module {
                expected: expected.module,
                got: module,
            });
        }
        if name != expected.name {
            return Err(Name {
                expected: expected.name,
                got: name,
            });
        }
        if !type_params.is_empty() {
            return Err(TypeParams(TypeParamsError::Number {
                expected: 0,
                got: type_params.len(),
            }));
        }
        Ok(Self)
    }
}

impl FromStr for StringTypeTag {
    type Err = ParseStructTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stag: StructTag = s.parse()?;
        Ok(stag.try_into()?)
    }
}

impl MoveType for String {
    type TypeTag = StringTypeTag;
}

impl MoveStruct for String {
    type StructTag = StringTypeTag;
}

impl StaticTypeTag for String {
    fn type_() -> Self::TypeTag {
        StringTypeTag {}
    }
}

impl StaticAddress for String {
    fn address() -> Address {
        Address::new(af_sui_types::hex_address_bytes(b"0x1"))
    }
}

impl StaticModule for String {
    fn module() -> Identifier {
        IdentStr::cast("string").to_owned()
    }
}

impl StaticName for String {
    fn name() -> Identifier {
        IdentStr::cast("String").to_owned()
    }
}

impl StaticTypeParams for String {
    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}
