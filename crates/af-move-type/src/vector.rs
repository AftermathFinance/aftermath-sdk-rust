use af_sui_types::TypeTag;
use derive_more::{Deref, DerefMut, From, Into};
use derive_where::derive_where;
use serde::{Deserialize, Serialize};

use crate::{MoveType, ParseTypeTagError, StaticTypeTag, TypeTagError};

#[derive(
    Clone, Debug, Deref, DerefMut, Deserialize, From, Into, Serialize, PartialEq, Eq, Hash,
)]
#[serde(bound(deserialize = ""))]
pub struct MoveVec<T: MoveType>(Vec<T>);

impl<T: MoveType> MoveType for MoveVec<T> {
    type TypeTag = VecTypeTag<T>;
}

impl<T: StaticTypeTag> StaticTypeTag for MoveVec<T> {
    fn type_() -> Self::TypeTag {
        VecTypeTag(T::type_())
    }
}

impl<T: MoveType> std::fmt::Display for MoveVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use tabled::settings::style::Style;
        use tabled::settings::{Rotate, Settings};
        use tabled::Table;

        let mut table = Table::from_iter([self.iter().map(|e| e.to_string())]);
        let settings = Settings::default()
            .with(Rotate::Right)
            .with(Style::rounded().remove_horizontals());
        table.with(settings);
        write!(f, "{table}")
    }
}

impl<T: MoveType> std::fmt::Display for crate::MoveInstance<MoveVec<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[derive_where(PartialOrd, Ord)]
pub struct VecTypeTag<T: MoveType>(T::TypeTag);

impl<T: MoveType> From<VecTypeTag<T>> for TypeTag {
    fn from(value: VecTypeTag<T>) -> Self {
        Self::Vector(Box::new(value.0.into()))
    }
}

impl<T: MoveType> TryFrom<TypeTag> for VecTypeTag<T> {
    type Error = TypeTagError;

    fn try_from(value: TypeTag) -> Result<Self, Self::Error> {
        match value {
            TypeTag::Vector(type_) => Ok(Self((*type_).try_into()?)),
            _ => Err(TypeTagError::Variant {
                expected: "Vector(_)".to_owned(),
                got: value,
            }),
        }
    }
}

impl<T: MoveType> std::str::FromStr for VecTypeTag<T> {
    type Err = ParseTypeTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag: TypeTag = s.parse()?;
        Ok(tag.try_into()?)
    }
}
