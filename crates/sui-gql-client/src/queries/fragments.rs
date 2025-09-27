use af_sui_types::{Address as SuiAddress, ObjectId, TypeTag, Version};
use cynic::QueryFragment;
use sui_gql_schema::{scalars, schema};

// ====================================================================================================
//  Input objects
// ====================================================================================================

/// This is only used in `Query.multiGetObjects` currently
#[derive(cynic::InputObject, Clone, Debug)]
pub struct ObjectKey {
    pub object_id: ObjectId,
    pub version: Version,
}

#[derive(cynic::InputObject, Clone, Debug, Default)]
#[cynic(graphql_type = "ObjectFilter")]
pub(crate) struct ObjectFilterV2<'a> {
    /// Filter objects by their type's `package`, `package::module`, or their fully qualified type
    /// name.
    ///
    /// Generic types can be queried by either the generic type name, e.g. `0x2::coin::Coin`, or by
    /// the full type name, such as `0x2::coin::Coin<0x2::sui::SUI>`.
    #[cynic(rename = "type")]
    pub(crate) type_: Option<String>,
    pub(crate) owner: Option<SuiAddress>,
    pub(crate) object_ids: Option<&'a [ObjectId]>,
}

#[derive(cynic::InputObject, Clone, Debug)]
pub struct DynamicFieldName {
    /// The string type of the DynamicField's 'name' field.
    /// A string representation of a Move primitive like 'u64', or a struct type like '0x2::kiosk::Listing'
    #[cynic(rename = "type")]
    pub type_: scalars::TypeTag,
    /// The Base64 encoded bcs serialization of the DynamicField's 'name' field.
    pub bcs: scalars::Base64<Vec<u8>>,
}

#[cfg(feature = "move-type")]
impl<T: af_move_type::MoveType> TryFrom<af_move_type::MoveInstance<T>> for DynamicFieldName {
    type Error = bcs::Error;

    fn try_from(value: af_move_type::MoveInstance<T>) -> Result<Self, Self::Error> {
        let af_move_type::MoveInstance { type_, value } = value;
        Ok(Self {
            type_: scalars::TypeTag(type_.into()),
            bcs: scalars::Base64::new(bcs::to_bytes(&value)?),
        })
    }
}

#[derive(cynic::InputObject, Clone, Debug, Default)]
pub(crate) struct TransactionBlockFilter {
    pub(crate) function: Option<String>,
    pub(crate) kind: Option<TransactionBlockKindInput>,
    pub(crate) after_checkpoint: Option<Version>,
    pub(crate) at_checkpoint: Option<Version>,
    pub(crate) before_checkpoint: Option<Version>,
    pub(crate) affected_address: Option<SuiAddress>,
    pub(crate) sent_address: Option<SuiAddress>,
    pub(crate) input_object: Option<SuiAddress>,
    pub(crate) changed_object: Option<SuiAddress>,
    pub(crate) transaction_ids: Option<Vec<String>>,
}

#[derive(cynic::Enum, Clone, Debug)]
pub(crate) enum TransactionBlockKindInput {
    SystemTx,
    ProgrammableTx,
}

// ====================================================================================================
//  Simple fragments
// ====================================================================================================

#[derive(cynic::QueryFragment, Clone, Debug, Default)]
pub(crate) struct PageInfo {
    pub(crate) has_next_page: bool,
    pub(crate) end_cursor: Option<String>,
    #[expect(dead_code, reason = "For generality")]
    pub(crate) has_previous_page: bool,
    #[expect(dead_code, reason = "For generality")]
    pub(crate) start_cursor: Option<String>,
}

impl From<PageInfoForward> for PageInfo {
    fn from(
        PageInfoForward {
            has_next_page,
            end_cursor,
        }: PageInfoForward,
    ) -> Self {
        Self {
            has_next_page,
            end_cursor,
            ..Default::default()
        }
    }
}

impl From<PageInfoBackward> for PageInfo {
    fn from(
        PageInfoBackward {
            has_previous_page,
            start_cursor,
        }: PageInfoBackward,
    ) -> Self {
        Self {
            has_previous_page,
            start_cursor,
            ..Default::default()
        }
    }
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfoForward {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfoBackward {
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "MoveValue")]
pub struct MoveValueRaw {
    #[cynic(rename = "type")]
    pub type_: MoveTypeTag,
    pub bcs: scalars::Base64<Vec<u8>>,
}

impl From<MoveValueRaw> for super::outputs::RawMoveValue {
    fn from(MoveValueRaw { type_, bcs }: MoveValueRaw) -> Self {
        Self {
            type_: type_.into(),
            bcs: bcs.into_inner(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("TypeTag is not a Struct variant")]
pub struct NotMoveStructError;

impl TryFrom<MoveValueRaw> for super::outputs::RawMoveStruct {
    type Error = NotMoveStructError;
    fn try_from(MoveValueRaw { type_, bcs }: MoveValueRaw) -> Result<Self, Self::Error> {
        let tag: TypeTag = type_.into();
        let TypeTag::Struct(stag) = tag else {
            return Err(NotMoveStructError);
        };
        Ok(Self {
            type_: *stag,
            bcs: bcs.into_inner(),
        })
    }
}

#[cfg(feature = "move-type")]
impl<T> TryFrom<MoveValueRaw> for af_move_type::MoveInstance<T>
where
    T: af_move_type::MoveType,
{
    type Error = ToMoveInstanceError;
    fn try_from(MoveValueRaw { bcs, type_ }: MoveValueRaw) -> Result<Self, Self::Error> {
        // Fail early if type tag is not expected
        let type_ = TypeTag::from(type_).try_into()?;
        let value = bcs::from_bytes(bcs.as_ref())?;
        Ok(Self { type_, value })
    }
}

#[cfg(feature = "move-type")]
#[derive(thiserror::Error, Debug)]
pub enum ToMoveInstanceError {
    #[error("Mismatched types: {0}")]
    TypeTag(#[from] af_move_type::TypeTagError),
    #[error("Deserializing value: {0}")]
    Bcs(#[from] bcs::Error),
}

/// Helper to extract a strongly typed [`TypeTag`] from the `MoveType` GQL type.
#[derive(cynic::QueryFragment, Clone)]
#[cynic(graphql_type = "MoveType")]
pub struct MoveTypeTag {
    /// Keep this private so that we can change where we get the [TypeTag] from.
    repr: scalars::TypeTag,
}

impl std::fmt::Debug for MoveTypeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MoveTypeTag({})", self.repr.0)
    }
}

impl From<MoveTypeTag> for TypeTag {
    fn from(value: MoveTypeTag) -> Self {
        value.repr.0
    }
}

// ====================================================================================================
//  Internal
// ====================================================================================================

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "MoveObject")]
pub(super) struct MoveObjectContent<MoveValue = MoveValueRaw>
where
    MoveValue: QueryFragment<SchemaType = schema::MoveValue, VariablesFields = ()>,
{
    pub(super) contents: Option<MoveValue>,
}

impl<MoveValue> MoveObjectContent<MoveValue>
where
    MoveValue: QueryFragment<SchemaType = schema::MoveValue, VariablesFields = ()>,
{
    pub fn into_content(self) -> Option<MoveValue> {
        self.contents
    }
}
