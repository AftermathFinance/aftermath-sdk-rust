//! Permissive type marker that accepts any Move type in its slot.

use std::str::FromStr;

use af_sui_types::TypeTag;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{MoveType, ParseTypeTagError, TypeTagError};

/// Generic type that accepts **any** Move type argument in its slot,
/// including phantom-paramed generics such as `VENDOR<X>` as well as
/// non-struct types (primitives, vectors).
///
/// Where [`crate::otw::Otw`] requires `n_types_expected == 0` and therefore
/// rejects phantom-paramed arguments, `AnyT` performs no validation — its
/// associated [`AnyTTypeTag`] simply captures the raw [`TypeTag`] so it
/// can be read back unchanged.
///
/// `AnyT` is intended for **phantom slots only** in generic Move types
/// (e.g. `AuthorityCap<AnyT, AnyT>`). It is never instantiated as a real
/// value; its [`MoveType`] impl exists so the derive-generated parsers can
/// thread phantom-paramed type arguments through unchanged.
///
/// Note: `AnyT` deliberately does **not** implement [`MoveStruct`] —
/// `MoveStructTag: TryFrom<StructTag>` is incompatible with carrying an
/// arbitrary [`TypeTag`] (which may be non-struct). Use [`crate::otw::Otw`]
/// when you need a `MoveStruct`-bounded slot type.
///
/// Unlike `Otw`, `AnyT` does **not** implement [`crate::StaticTypeTag`] or
/// its `Static*` siblings: it has no statically known type tag, by design.
/// Use it on the runtime decode path (e.g.
/// [`MoveInstance::from_raw_type`](crate::MoveInstance::from_raw_type)).
///
/// [`MoveStruct`]: crate::MoveStruct
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct AnyT {
    dummy_field: bool,
}

impl AnyT {
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for AnyT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnyT")
    }
}

/// `TypeTag` companion for [`AnyT`]. Wraps the raw [`TypeTag`] from the slot
/// with **no** validation of variant, address, module, name, or type-param
/// count.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AnyTTypeTag(pub TypeTag);

impl From<AnyTTypeTag> for TypeTag {
    fn from(value: AnyTTypeTag) -> Self {
        value.0
    }
}

impl TryFrom<TypeTag> for AnyTTypeTag {
    type Error = TypeTagError;

    fn try_from(value: TypeTag) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl FromStr for AnyTTypeTag {
    type Err = ParseTypeTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag: TypeTag = s.parse()?;
        Ok(Self(tag))
    }
}

impl std::fmt::Display for AnyTTypeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for AnyTTypeTag {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for AnyTTypeTag {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let s = String::deserialize(de)?;
        s.parse().map_err(D::Error::custom)
    }
}

impl MoveType for AnyT {
    type TypeTag = AnyTTypeTag;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `AnyTTypeTag` accepts a struct tag whose own type params carry
    /// phantom-paramed structs — the exact case `Otw` rejects.
    #[test]
    fn accepts_phantom_paramed_struct_tag() {
        let raw: TypeTag = "0x1::authority::AuthorityCap<0x1::authority::VENDOR<0x2::aftermath::AFTERMATH>, 0x1::authority::ASSISTANT>"
            .parse()
            .unwrap();
        let wrapped = AnyTTypeTag::try_from(raw.clone()).unwrap();
        let back: TypeTag = wrapped.into();
        assert_eq!(back, raw);
    }

    /// `AnyTTypeTag` is permissive over `TypeTag` variants — not just
    /// `Struct(_)` — unlike a `MoveStruct`'s derived companion.
    #[test]
    fn accepts_non_struct_type_tag() {
        for raw in [TypeTag::U64, TypeTag::Bool, TypeTag::Address] {
            let wrapped = AnyTTypeTag::try_from(raw.clone()).unwrap();
            assert_eq!(TypeTag::from(wrapped), raw);
        }
    }

    /// `Display` / `FromStr` round-trip through the canonical TypeTag form.
    #[test]
    fn display_fromstr_roundtrip() {
        let raw: TypeTag = "0x1::a::B<0x2::c::D<0x3::e::F>>".parse().unwrap();
        let wrapped = AnyTTypeTag(raw.clone());
        let s = wrapped.to_string();
        let parsed = AnyTTypeTag::from_str(&s).unwrap();
        assert_eq!(parsed.0, raw);
    }

    /// `Serialize` / `Deserialize` round-trip via JSON.
    #[test]
    fn serde_roundtrip_json() {
        let raw: TypeTag = "0x1::a::B<0x2::c::D<0x3::e::F>>".parse().unwrap();
        let wrapped = AnyTTypeTag(raw.clone());
        let json = serde_json::to_string(&wrapped).unwrap();
        let back: AnyTTypeTag = serde_json::from_str(&json).unwrap();
        assert_eq!(back.0, raw);
    }
}
