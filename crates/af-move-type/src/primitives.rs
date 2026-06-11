use std::str::FromStr;

use af_sui_types::{Address, TypeTag, U256};
use serde::{Deserialize, Serialize};

use crate::{MoveType, MoveTypeTag, ParseTypeTagError, StaticTypeTag, TypeTagError};

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

            impl MoveTypeTag for $type_ {
                fn matches(tag: &TypeTag) -> bool {
                    matches!(tag, TypeTag::$variant)
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
