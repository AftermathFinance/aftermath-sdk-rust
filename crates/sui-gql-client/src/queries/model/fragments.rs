use af_sui_types::{
    Address as SuiAddress,
    Object as ObjectSdk,
    Transaction as TransactionSdk,
    TypeTag,
    Version,
};
use cynic::QueryFragment;
use enum_as_inner::EnumAsInner;
use sui_gql_schema::scalars::{self, BigInt};
use sui_gql_schema::schema;

// ====================================================================================================
//  Query Input Fragments
// ====================================================================================================

/// This is only used in `Query.multiGetObjects` currently
#[derive(cynic::InputObject, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectKey {
    pub address: SuiAddress,
    pub version: Option<Version>,
    pub root_version: Option<Version>,
    pub at_checkpoint: Option<u64>,
}

#[derive(cynic::InputObject, Clone, Debug, Default)]
#[cynic(graphql_type = "ObjectFilter")]
pub(crate) struct ObjectFilter {
    /// Filter objects by their type's `package`, `package::module`, or their fully qualified type
    /// name.
    ///
    /// Generic types can be queried by either the generic type name, e.g. `0x2::coin::Coin`, or by
    /// the full type name, such as `0x2::coin::Coin<0x2::sui::SUI>`.
    #[cynic(rename = "type")]
    pub(crate) type_: Option<String>,
    pub(crate) owner: Option<SuiAddress>,
    pub(crate) owner_kind: Option<OwnerKind>,
}

#[derive(Clone, Debug, cynic::Enum)]
pub enum OwnerKind {
    Address,
    Object,
    Shared,
    Immutable,
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
pub struct TransactionFilter {
    pub function: Option<String>,
    pub kind: Option<TransactionKindInput>,
    pub after_checkpoint: Option<Version>,
    pub at_checkpoint: Option<Version>,
    pub before_checkpoint: Option<Version>,
    pub affected_address: Option<SuiAddress>,
    pub sent_address: Option<SuiAddress>,
    pub affected_object: Option<SuiAddress>,
}

#[derive(cynic::Enum, Clone, Debug)]
pub enum TransactionKindInput {
    SystemTx,
    ProgrammableTx,
}

// ====================================================================================================
//  PageInfo Fragments
// ====================================================================================================

#[derive(cynic::QueryFragment, Clone, Debug, Default)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
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

// =============================================================================
//  Inner Fragments
// =============================================================================

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "MoveValue")]
pub struct MoveValueGql {
    #[cynic(rename = "type")]
    pub type_: Option<MoveTypeGql>,
    pub bcs: Option<scalars::Base64<Vec<u8>>>,
}

/// `ObjectConnection` where the `Object` fragment does take any parameters.
#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct ObjectConnection {
    pub nodes: Vec<ObjectGql>,
    pub page_info: PageInfoForward,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Object")]
pub struct ObjectGql {
    #[cynic(rename = "address")]
    pub id: SuiAddress,
    #[cynic(rename = "objectBcs")]
    pub object: Option<scalars::Base64Bcs<ObjectSdk>>,
}

impl TryFrom<MoveValueGql> for super::outputs::RawMoveValue {
    type Error = TryFromMoveValue;
    fn try_from(MoveValueGql { type_, bcs }: MoveValueGql) -> Result<Self, Self::Error> {
        let (Some(type_), Some(bcs)) = (type_, bcs) else {
            return Err(TryFromMoveValue::MissingData);
        };

        Ok(Self {
            type_: type_.into(),
            bcs: bcs.into_inner(),
        })
    }
}

impl TryFrom<MoveValueGql> for super::outputs::RawMoveStruct {
    type Error = TryFromMoveValue;
    fn try_from(MoveValueGql { type_, bcs }: MoveValueGql) -> Result<Self, Self::Error> {
        let (Some(type_), Some(bcs)) = (type_, bcs) else {
            return Err(TryFromMoveValue::MissingData);
        };
        let tag: TypeTag = type_.into();
        let TypeTag::Struct(stag) = tag else {
            return Err(TryFromMoveValue::NotMoveStructError);
        };

        Ok(Self {
            type_: *stag,
            bcs: bcs.into_inner(),
        })
    }
}

impl<T> TryFrom<MoveValueGql> for af_move_type::MoveInstance<T>
where
    T: af_move_type::MoveType,
{
    type Error = ToMoveInstanceError;
    fn try_from(MoveValueGql { bcs, type_ }: MoveValueGql) -> Result<Self, Self::Error> {
        let (Some(type_), Some(bcs)) = (type_, bcs) else {
            return Err(TryFromMoveValue::MissingData.into());
        };
        // Fail early if type tag is not expected
        let type_ = TypeTag::from(type_).try_into()?;
        let value = bcs::from_bytes(bcs.as_ref())?;
        Ok(Self { type_, value })
    }
}

/// Helper to extract a strongly typed [`TypeTag`] from the `MoveType` GQL type.
#[derive(cynic::QueryFragment, Clone)]
#[cynic(graphql_type = "MoveType")]
pub struct MoveTypeGql {
    /// Keep this private so that we can change where we get the [TypeTag] from.
    repr: scalars::TypeTag,
}

impl std::fmt::Debug for MoveTypeGql {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MoveTypeTag({})", self.repr.0)
    }
}

impl From<MoveTypeGql> for TypeTag {
    fn from(value: MoveTypeGql) -> Self {
        value.repr.0
    }
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DynamicField")]
pub struct DynamicFieldByName {
    pub value: Option<DynamicFieldValue>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DynamicFieldConnection {
    pub nodes: Vec<DynamicField>,
    pub page_info: PageInfo,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DynamicField {
    pub name: Option<MoveValueGql>,
    pub value: Option<DynamicFieldValue>,
}

#[derive(cynic::InlineFragments, Debug, EnumAsInner)]
pub enum DynamicFieldValue {
    MoveObject(MoveObject),
    MoveValue(MoveValueGql),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct MoveObject {
    pub address: SuiAddress,
    pub version: Option<Version>,
    pub contents: Option<MoveValueGql>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Checkpoint {
    pub sequence_number: af_sui_types::Version,
}

#[derive(QueryFragment, Clone, Debug)]
pub struct Epoch {
    pub epoch_id: Version,
    pub reference_gas_price: Option<BigInt<u64>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Transaction")]
pub struct TransactionGql {
    pub digest: String,
    #[cynic(rename = "transactionBcs")]
    pub bcs: Option<scalars::Base64Bcs<TransactionSdk>>,
    pub effects: Option<TransactionEffects>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct TransactionEffects {
    pub status: Option<ExecutionStatus>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum ExecutionStatus {
    Success,
    Failure,
}

impl From<ExecutionStatus> for bool {
    fn from(value: ExecutionStatus) -> Self {
        match value {
            ExecutionStatus::Success => true,
            ExecutionStatus::Failure => false,
        }
    }
}

// ====================================================================================================
//  Events
// ====================================================================================================

#[derive(cynic::InputObject, Debug, Clone)]
pub struct EventFilter {
    pub sender: Option<SuiAddress>,
    pub after_checkpoint: Option<u64>,
    pub before_checkpoint: Option<u64>,
    pub at_checkpoint: Option<u64>,
    #[cynic(rename = "type")]
    pub type_: Option<String>,
    pub module: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct EventEdge {
    pub node: Event,
    pub cursor: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Event {
    pub timestamp: Option<scalars::DateTime>,
    pub contents: Option<MoveValueGql>,
}

// ====================================================================================================
//  Packages
// ====================================================================================================

#[derive(cynic::QueryFragment, Debug)]
pub struct MovePackageConnection {
    pub nodes: Vec<MovePackage>,
    pub page_info: PageInfoForward,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct MovePackage {
    pub address: SuiAddress,
    pub version: Option<Version>,
}

// ====================================================================================================
//  Errors
// ====================================================================================================

#[derive(thiserror::Error, Debug)]
pub enum ToMoveInstanceError {
    #[error("Mismatched types: {0}")]
    TypeTag(#[from] af_move_type::TypeTagError),
    #[error("Deserializing value: {0}")]
    Bcs(#[from] bcs::Error),
    #[error("Deserializing value: {0}")]
    TryFromMoveValue(#[from] TryFromMoveValue),
}

#[derive(thiserror::Error, Debug)]
pub enum TryFromMoveValue {
    #[error("Either type or bcs are missing")]
    MissingData,

    #[error("TypeTag is not a Struct variant")]
    NotMoveStructError,
}
