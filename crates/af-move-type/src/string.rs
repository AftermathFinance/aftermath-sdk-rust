use std::str::FromStr;

use af_sui_types::{Address, IdentStr, Identifier, StructTag, TypeTag};
use serde::{Deserialize, Serialize};

use crate::{
    MoveStruct,
    MoveType,
    ParseStructTagError,
    StaticAddress,
    StaticModule,
    StaticName,
    StaticStructTag as _,
    StaticTypeParams,
    StaticTypeTag,
    StructTagError,
    TypeParamsError,
    TypeTagError,
};

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
