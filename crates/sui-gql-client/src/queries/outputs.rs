use af_sui_types::{encode_base64_default, ObjectId, StructTag, TypeTag};
use derive_more::Display;

/// An instance of a dynamic field or dynamic object.
#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicField {
    /// The object key (id, version) and the raw struct contents.
    ///
    /// The object key is useful to query further data about the object later.
    /// It is also guaranteed that the contents are of a struct, since enums cannot be top-level
    /// objects.
    ///
    /// Reference: <https://move-book.com/reference/enums.html>
    #[display("DOF {{\n\t{_0}: {_1}\n}})")]
    Object(ObjectKey, RawMoveStruct),
    /// The raw Move value contents. Could be a primitive type, struct, or enum.
    #[display("DF {{\n\t{_0}\n}})")]
    Field(RawMoveValue),
}

/// Reference to a specific object instance in time.
#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display("({object_id}, {version})")]
pub struct ObjectKey {
    pub object_id: ObjectId,
    pub version: u64,
}

/// Raw representation of a Move value
#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display("{type_} {{\n\t{}\n}}", encode_base64_default(bcs))]
pub struct RawMoveValue {
    pub type_: TypeTag,
    pub bcs: Vec<u8>,
}

#[cfg(feature = "move-type")]
impl<T: af_move_type::MoveType> TryFrom<RawMoveValue> for af_move_type::MoveInstance<T> {
    type Error = af_move_type::FromRawTypeError;
    fn try_from(RawMoveValue { type_, bcs }: RawMoveValue) -> Result<Self, Self::Error> {
        Self::from_raw_type(type_, &bcs)
    }
}

#[cfg(feature = "move-type")]
impl<T: af_move_type::MoveType> TryFrom<af_move_type::MoveInstance<T>> for RawMoveValue {
    type Error = bcs::Error;
    fn try_from(value: af_move_type::MoveInstance<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            type_: value.type_.into(),
            bcs: value.value.to_bcs()?,
        })
    }
}

/// Raw representation of a Move struct
#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display("{type_} {{\n\t{}\n}}", encode_base64_default(bcs))]
pub struct RawMoveStruct {
    pub type_: StructTag,
    pub bcs: Vec<u8>,
}

#[cfg(feature = "move-type")]
impl<T: af_move_type::MoveStruct> TryFrom<RawMoveStruct> for af_move_type::MoveInstance<T> {
    type Error = af_move_type::FromRawStructError;
    fn try_from(RawMoveStruct { type_, bcs }: RawMoveStruct) -> Result<Self, Self::Error> {
        Self::from_raw_struct(type_, &bcs)
    }
}

#[cfg(feature = "move-type")]
impl<T: af_move_type::MoveStruct> TryFrom<af_move_type::MoveInstance<T>> for RawMoveStruct {
    type Error = bcs::Error;
    fn try_from(value: af_move_type::MoveInstance<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            type_: value.type_.into(),
            bcs: value.value.to_bcs()?,
        })
    }
}
