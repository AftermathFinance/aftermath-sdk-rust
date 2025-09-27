// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

use af_sui_types::{
    Address as SuiAddress,
    Identifier,
    MoveObjectType,
    Object,
    ObjectArg,
    ObjectDigest,
    ObjectId,
    ObjectRef,
    StructTag,
    TransactionDigest,
    TypeOrigin,
    UpgradeInfo,
};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::base64::Base64;
use serde_with::{DisplayFromStr, serde_as};
use sui_sdk_types::Version;

use super::{Page, SuiMoveStruct, SuiMoveValue};
use crate::serde::BigInt;

// =============================================================================
//  SuiObjectResponse
// =============================================================================

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
#[error("Could not get object_id, something went wrong with SuiObjectResponse construction.")]
pub struct MissingObjectIdError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SuiObjectResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<SuiObjectData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SuiObjectResponseError>,
}

impl SuiObjectResponse {
    pub fn new(data: Option<SuiObjectData>, error: Option<SuiObjectResponseError>) -> Self {
        Self { data, error }
    }

    pub fn new_with_data(data: SuiObjectData) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    pub fn new_with_error(error: SuiObjectResponseError) -> Self {
        Self {
            data: None,
            error: Some(error),
        }
    }

    /// Returns a reference to the object if there is any, otherwise an Err if
    /// the object does not exist or is deleted.
    pub fn object(&self) -> Result<&SuiObjectData, SuiObjectResponseError> {
        if let Some(data) = &self.data {
            Ok(data)
        } else if let Some(error) = &self.error {
            Err(error.clone())
        } else {
            // We really shouldn't reach this code block since either data, or error field should always be filled.
            Err(SuiObjectResponseError::Unknown)
        }
    }

    /// Returns the object value if there is any, otherwise an Err if
    /// the object does not exist or is deleted.
    pub fn into_object(self) -> Result<SuiObjectData, SuiObjectResponseError> {
        match self.object() {
            Ok(data) => Ok(data.clone()),
            Err(error) => Err(error),
        }
    }

    pub fn move_object_bcs(&self) -> Option<&Vec<u8>> {
        match &self.data {
            Some(SuiObjectData {
                bcs: Some(SuiRawData::MoveObject(obj)),
                ..
            }) => Some(&obj.bcs_bytes),
            _ => None,
        }
    }

    pub fn owner(&self) -> Option<Owner> {
        if let Some(data) = &self.data {
            return data.owner.clone();
        }
        None
    }

    pub fn object_id(&self) -> Result<ObjectId, MissingObjectIdError> {
        match (&self.data, &self.error) {
            (Some(obj_data), None) => Ok(obj_data.object_id),
            (None, Some(SuiObjectResponseError::NotExists { object_id })) => Ok(*object_id),
            (
                None,
                Some(SuiObjectResponseError::Deleted {
                    object_id,
                    version: _,
                    digest: _,
                }),
            ) => Ok(*object_id),
            _ => Err(MissingObjectIdError),
        }
    }

    pub fn object_ref_if_exists(&self) -> Option<ObjectRef> {
        match (&self.data, &self.error) {
            (Some(obj_data), None) => Some(obj_data.object_ref()),
            _ => None,
        }
    }
}

impl Ord for SuiObjectResponse {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.data, &other.data) {
            (Some(data), Some(data_2)) => {
                if data.object_id.cmp(&data_2.object_id).eq(&Ordering::Greater) {
                    return Ordering::Greater;
                } else if data.object_id.cmp(&data_2.object_id).eq(&Ordering::Less) {
                    return Ordering::Less;
                }
                Ordering::Equal
            }
            // In this ordering those with data will come before SuiObjectResponses that are errors.
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            // SuiObjectResponses that are errors are just considered equal.
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for SuiObjectResponse {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Originally from `sui_types::error`.
#[derive(thiserror::Error, Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Hash)]
#[serde(tag = "code", rename = "ObjectResponseError", rename_all = "camelCase")]
pub enum SuiObjectResponseError {
    #[error("Object {:?} does not exist.", object_id)]
    NotExists { object_id: ObjectId },
    #[error("Cannot find dynamic field for parent object {:?}.", parent_object_id)]
    DynamicFieldNotFound { parent_object_id: ObjectId },
    #[error(
        "Object has been deleted object_id: {:?} at version: {:?} in digest {:?}",
        object_id,
        version,
        digest
    )]
    Deleted {
        object_id: ObjectId,
        /// Object version.
        version: Version,
        /// Base64 string representing the object digest
        digest: ObjectDigest,
    },
    #[error("Unknown Error.")]
    Unknown,
    #[error("Display Error: {:?}", error)]
    DisplayError { error: String },
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct DisplayFieldsResponse {
    pub data: Option<BTreeMap<String, String>>,
    pub error: Option<SuiObjectResponseError>,
}

// =============================================================================
//  SuiObjectData
// =============================================================================

#[derive(thiserror::Error, Debug)]
pub enum SuiObjectDataError {
    #[error("Missing object type")]
    MissingObjectType,
    #[error("Missing BCS encoding")]
    MissingBcs,
    #[error("Missing object owner")]
    MissingOwner,
    #[error("Not a Move object")]
    NotMoveObject,
    #[error("Not an immutable or owned object")]
    NotImmOrOwned,
    #[error("Not a shared object")]
    NotShared,
    #[error(transparent)]
    ObjectType(#[from] NotMoveStructError),
}

/// Error for [`SuiObjectData::into_full_object`].
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum FullObjectDataError {
    #[error("Missing BCS encoding")]
    MissingBcs,
    #[error("Missing object owner")]
    MissingOwner,
    #[error("Missing previous transaction digest")]
    MissingPreviousTransaction,
    #[error("Missing storage rebate")]
    MissingStorageRebate,
    #[error("MoveObject BCS doesn't start with ObjectId")]
    InvalidBcs,
    #[error("Invalid identifier: {ident}\nReason: {source}")]
    InvalidIdentifier {
        ident: Box<str>,
        #[source]
        source: af_sui_types::TypeParseError,
    },
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", rename = "ObjectData")]
pub struct SuiObjectData {
    pub object_id: ObjectId,
    /// Object version.
    #[serde_as(as = "BigInt<u64>")]
    pub version: Version,
    /// Base64 string representing the object digest
    pub digest: ObjectDigest,
    /// The type of the object. Default to be None unless SuiObjectDataOptions.showType is set to true
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<ObjectType>,
    // Default to be None because otherwise it will be repeated for the getOwnedObjects endpoint
    /// The owner of this object. Default to be None unless SuiObjectDataOptions.showOwner is set to true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,
    /// The digest of the transaction that created or last mutated this object. Default to be None unless
    /// SuiObjectDataOptions.showPreviousTransaction is set to true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_transaction: Option<TransactionDigest>,
    /// The amount of SUI we would rebate if this object gets deleted.
    /// This number is re-calculated each time the object is mutated based on
    /// the present storage gas price.
    #[serde_as(as = "Option<BigInt<u64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_rebate: Option<u64>,
    /// The Display metadata for frontend UI rendering, default to be None unless SuiObjectDataOptions.showContent is set to true
    /// This can also be None if the struct type does not have Display defined
    /// See more details in <https://forums.sui.io/t/nft-object-display-proposal/4872>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<DisplayFieldsResponse>,
    /// Move object content or package content, default to be None unless SuiObjectDataOptions.showContent is set to true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<SuiParsedData>,
    /// Move object content or package content in BCS, default to be None unless SuiObjectDataOptions.showBcs is set to true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcs: Option<SuiRawData>,
}

impl SuiObjectData {
    pub fn object_ref(&self) -> ObjectRef {
        (self.object_id, self.version, self.digest)
    }

    pub fn object_type(&self) -> Result<ObjectType, SuiObjectDataError> {
        self.type_
            .as_ref()
            .ok_or(SuiObjectDataError::MissingObjectType)
            .cloned()
    }

    pub fn is_gas_coin(&self) -> bool {
        match self.type_.as_ref() {
            Some(ObjectType::Struct(ty)) if ty.is_gas_coin() => true,
            Some(_) => false,
            None => false,
        }
    }

    pub fn struct_tag(&self) -> Result<StructTag, SuiObjectDataError> {
        Ok(self
            .type_
            .clone()
            .ok_or(SuiObjectDataError::MissingObjectType)?
            .try_into()?)
    }

    pub fn take_object_type(&mut self) -> Result<ObjectType, SuiObjectDataError> {
        self.type_
            .take()
            .ok_or(SuiObjectDataError::MissingObjectType)
    }

    pub fn take_raw_object(&mut self) -> Result<SuiRawMoveObject, SuiObjectDataError> {
        self.take_raw_data()?
            .try_into_move()
            .ok_or(SuiObjectDataError::NotMoveObject)
    }

    pub fn take_raw_data(&mut self) -> Result<SuiRawData, SuiObjectDataError> {
        self.bcs.take().ok_or(SuiObjectDataError::MissingBcs)
    }

    pub fn shared_object_arg(&self, mutable: bool) -> Result<ObjectArg, SuiObjectDataError> {
        let Owner::Shared {
            initial_shared_version,
        } = self.owner()?
        else {
            return Err(SuiObjectDataError::NotShared);
        };
        Ok(ObjectArg::SharedObject {
            id: self.object_id,
            initial_shared_version,
            mutable,
        })
    }

    pub fn imm_or_owned_object_arg(&self) -> Result<ObjectArg, SuiObjectDataError> {
        use Owner::*;
        if !matches!(self.owner()?, AddressOwner(_) | ObjectOwner(_) | Immutable) {
            return Err(SuiObjectDataError::NotImmOrOwned);
        };
        let (i, v, d) = self.object_ref();
        Ok(ObjectArg::ImmOrOwnedObject((i, v, d)))
    }

    #[cfg(feature = "client")]
    pub(crate) fn object_arg(&self, mutable: bool) -> Result<ObjectArg, SuiObjectDataError> {
        use Owner as O;
        Ok(match self.owner()? {
            O::AddressOwner(_) | O::ObjectOwner(_) | O::Immutable => {
                ObjectArg::ImmOrOwnedObject(self.object_ref())
            }
            O::Shared {
                initial_shared_version,
            }
            | O::ConsensusAddressOwner {
                start_version: initial_shared_version,
                ..
            } => ObjectArg::SharedObject {
                id: self.object_id,
                initial_shared_version,
                mutable,
            },
        })
    }

    pub fn owner(&self) -> Result<Owner, SuiObjectDataError> {
        self.owner.clone().ok_or(SuiObjectDataError::MissingOwner)
    }

    /// Create a standard Sui [`Object`] if there's enough information.
    pub fn into_full_object(self) -> Result<Object, FullObjectDataError> {
        use itertools::Itertools as _;
        let Self {
            owner,
            previous_transaction,
            storage_rebate,
            bcs,
            ..
        } = self;
        let owner = owner.ok_or(FullObjectDataError::MissingOwner)?;
        let previous_transaction =
            previous_transaction.ok_or(FullObjectDataError::MissingPreviousTransaction)?;
        let storage_rebate = storage_rebate.ok_or(FullObjectDataError::MissingStorageRebate)?;

        match bcs.ok_or(FullObjectDataError::MissingBcs)? {
            SuiRawData::Package(p) => {
                let modules = p
                    .module_map
                    .into_iter()
                    .map(|(s, bytes)| {
                        Ok((
                            s.parse()
                                .map_err(|e| FullObjectDataError::InvalidIdentifier {
                                    ident: s.into(),
                                    source: e,
                                })?,
                            bytes,
                        ))
                    })
                    .try_collect()?;
                let inner = af_sui_types::MovePackage {
                    id: p.id,
                    version: p.version,
                    modules,
                    type_origin_table: p.type_origin_table,
                    linkage_table: p.linkage_table,
                };
                Ok(Object::new(
                    sui_sdk_types::ObjectData::Package(inner),
                    owner.into(),
                    previous_transaction,
                    storage_rebate,
                ))
            }
            SuiRawData::MoveObject(raw_struct) => {
                let inner = sui_sdk_types::MoveStruct::new(
                    raw_struct.type_,
                    raw_struct.has_public_transfer,
                    raw_struct.version,
                    raw_struct.bcs_bytes,
                )
                .ok_or(FullObjectDataError::InvalidBcs)?;
                Ok(Object::new(
                    sui_sdk_types::ObjectData::Struct(inner),
                    owner.into(),
                    previous_transaction,
                    storage_rebate,
                ))
            }
        }
    }
}

impl Display for SuiObjectData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let type_ = if let Some(type_) = &self.type_ {
            type_.to_string()
        } else {
            "Unknown Type".into()
        };
        let mut writer = String::new();
        writeln!(
            writer,
            "{}",
            format!("----- {type_} ({}[{}]) -----", self.object_id, self.version).bold()
        )?;
        if let Some(ref owner) = self.owner {
            writeln!(writer, "{}: {:?}", "Owner".bold().bright_black(), owner)?;
        }

        writeln!(
            writer,
            "{}: {}",
            "Version".bold().bright_black(),
            self.version
        )?;
        if let Some(storage_rebate) = self.storage_rebate {
            writeln!(
                writer,
                "{}: {}",
                "Storage Rebate".bold().bright_black(),
                storage_rebate
            )?;
        }

        if let Some(previous_transaction) = self.previous_transaction {
            writeln!(
                writer,
                "{}: {:?}",
                "Previous Transaction".bold().bright_black(),
                previous_transaction
            )?;
        }
        if let Some(content) = self.content.as_ref() {
            writeln!(writer, "{}", "----- Data -----".bold())?;
            write!(writer, "{}", content)?;
        }

        write!(f, "{}", writer)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash, Ord, PartialOrd)]
pub enum Owner {
    /// Object is exclusively owned by a single address, and is mutable.
    AddressOwner(SuiAddress),
    /// Object is exclusively owned by a single object, and is mutable.
    /// The object ID is converted to SuiAddress as SuiAddress is universal.
    ObjectOwner(SuiAddress),
    /// Object is shared, can be used by any address, and is mutable.
    Shared {
        /// The version at which the object became shared
        initial_shared_version: Version,
    },
    /// Object is immutable, and hence ownership doesn't matter.
    Immutable,
    /// Object is exclusively owned by a single address and sequenced via consensus.
    ConsensusAddressOwner {
        /// The version at which the object most recently became a consensus object.
        /// This serves the same function as `initial_shared_version`, except it may change
        /// if the object's Owner type changes.
        start_version: Version,
        // The owner of the object.
        owner: SuiAddress,
    },
}

impl From<Owner> for sui_sdk_types::Owner {
    fn from(value: Owner) -> sui_sdk_types::Owner {
        match value {
            Owner::AddressOwner(a) => sui_sdk_types::Owner::Address(a),
            Owner::ObjectOwner(o) => sui_sdk_types::Owner::Object(o.into()),
            Owner::Shared {
                initial_shared_version,
            } => sui_sdk_types::Owner::Shared(initial_shared_version),
            Owner::Immutable => sui_sdk_types::Owner::Immutable,
            Owner::ConsensusAddressOwner {
                start_version,
                owner,
            } => sui_sdk_types::Owner::ConsensusAddress {
                start_version,
                owner,
            },
        }
    }
}
// =============================================================================
//  ObjectType
// =============================================================================

const PACKAGE: &str = "package";
/// Type of a Sui object
///
/// Originally from `sui_types::base_types`.
#[derive(Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum ObjectType {
    /// Move package containing one or more bytecode modules
    Package,
    /// A Move struct of the given type
    Struct(MoveObjectType),
}

impl TryFrom<ObjectType> for StructTag {
    type Error = NotMoveStructError;

    fn try_from(o: ObjectType) -> Result<Self, Self::Error> {
        match o {
            ObjectType::Package => Err(NotMoveStructError),
            ObjectType::Struct(move_object_type) => Ok(move_object_type.into()),
        }
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
#[error("Cannot create StructTag from Package")]
pub struct NotMoveStructError;

impl Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Package => write!(f, "{}", PACKAGE),
            ObjectType::Struct(t) => write!(f, "{}", t),
        }
    }
}

impl FromStr for ObjectType {
    type Err = <StructTag as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == PACKAGE {
            Ok(ObjectType::Package)
        } else {
            let tag: StructTag = s.parse()?;
            Ok(ObjectType::Struct(MoveObjectType::from(tag)))
        }
    }
}

// =============================================================================
//  SuiObjectDataOptions
// =============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase", rename = "ObjectDataOptions", default)]
pub struct SuiObjectDataOptions {
    /// Whether to show the type of the object. Default to be False
    pub show_type: bool,
    /// Whether to show the owner of the object. Default to be False
    pub show_owner: bool,
    /// Whether to show the previous transaction digest of the object. Default to be False
    pub show_previous_transaction: bool,
    /// Whether to show the Display metadata of the object for frontend rendering. Default to be False
    pub show_display: bool,
    /// Whether to show the content(i.e., package content or Move struct content) of the object.
    /// Default to be False
    pub show_content: bool,
    /// Whether to show the content in BCS format. Default to be False
    pub show_bcs: bool,
    /// Whether to show the storage rebate of the object. Default to be False
    pub show_storage_rebate: bool,
}

impl SuiObjectDataOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Just enough information to create an [`Object`].
    pub fn full_object() -> Self {
        Self {
            show_bcs: true,
            show_owner: true,
            show_storage_rebate: true,
            show_previous_transaction: true,
            show_content: false,
            show_display: false,
            show_type: false,
        }
    }

    /// return BCS data and all other metadata such as storage rebate
    pub fn bcs_lossless() -> Self {
        Self {
            show_bcs: true,
            show_type: true,
            show_owner: true,
            show_previous_transaction: true,
            show_display: false,
            show_content: false,
            show_storage_rebate: true,
        }
    }

    /// return full content except bcs
    pub fn full_content() -> Self {
        Self {
            show_bcs: false,
            show_type: true,
            show_owner: true,
            show_previous_transaction: true,
            show_display: false,
            show_content: true,
            show_storage_rebate: true,
        }
    }

    pub fn with_content(mut self) -> Self {
        self.show_content = true;
        self
    }

    pub fn with_owner(mut self) -> Self {
        self.show_owner = true;
        self
    }

    pub fn with_type(mut self) -> Self {
        self.show_type = true;
        self
    }

    pub fn with_display(mut self) -> Self {
        self.show_display = true;
        self
    }

    pub fn with_bcs(mut self) -> Self {
        self.show_bcs = true;
        self
    }

    pub fn with_previous_transaction(mut self) -> Self {
        self.show_previous_transaction = true;
        self
    }

    pub fn is_not_in_object_info(&self) -> bool {
        self.show_bcs || self.show_content || self.show_display || self.show_storage_rebate
    }
}

// =============================================================================
//  SuiObjectRef
// =============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase", rename = "ObjectRef")]
pub struct SuiObjectRef {
    /// Hex code as string representing the object id
    pub object_id: ObjectId,
    /// Object version.
    pub version: Version,
    /// Base64 string representing the object digest
    pub digest: ObjectDigest,
}

impl SuiObjectRef {
    pub fn to_object_ref(&self) -> ObjectRef {
        (self.object_id, self.version, self.digest)
    }
}

impl Display for SuiObjectRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Object ID: {}, version: {}, digest: {}",
            self.object_id, self.version, self.digest
        )
    }
}

impl From<ObjectRef> for SuiObjectRef {
    fn from(oref: ObjectRef) -> Self {
        Self {
            object_id: oref.0,
            version: oref.1,
            digest: oref.2,
        }
    }
}

// =============================================================================
//  SuiData
// =============================================================================

pub trait SuiData: Sized {
    type ObjectType;
    type PackageType;
    fn try_as_move(&self) -> Option<&Self::ObjectType>;
    fn try_into_move(self) -> Option<Self::ObjectType>;
    fn try_as_package(&self) -> Option<&Self::PackageType>;
    fn type_(&self) -> Option<&StructTag>;
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(tag = "dataType", rename_all = "camelCase", rename = "RawData")]
pub enum SuiRawData {
    // Manually handle generic schema generation
    MoveObject(SuiRawMoveObject),
    Package(SuiRawMovePackage),
}

impl SuiData for SuiRawData {
    type ObjectType = SuiRawMoveObject;
    type PackageType = SuiRawMovePackage;

    fn try_as_move(&self) -> Option<&Self::ObjectType> {
        match self {
            Self::MoveObject(o) => Some(o),
            Self::Package(_) => None,
        }
    }

    fn try_into_move(self) -> Option<Self::ObjectType> {
        match self {
            Self::MoveObject(o) => Some(o),
            Self::Package(_) => None,
        }
    }

    fn try_as_package(&self) -> Option<&Self::PackageType> {
        match self {
            Self::MoveObject(_) => None,
            Self::Package(p) => Some(p),
        }
    }

    fn type_(&self) -> Option<&StructTag> {
        match self {
            Self::MoveObject(o) => Some(&o.type_),
            Self::Package(_) => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(tag = "dataType", rename_all = "camelCase", rename = "Data")]
pub enum SuiParsedData {
    // Manually handle generic schema generation
    MoveObject(SuiParsedMoveObject),
    Package(SuiMovePackage),
}

impl SuiData for SuiParsedData {
    type ObjectType = SuiParsedMoveObject;
    type PackageType = SuiMovePackage;

    fn try_as_move(&self) -> Option<&Self::ObjectType> {
        match self {
            Self::MoveObject(o) => Some(o),
            Self::Package(_) => None,
        }
    }

    fn try_into_move(self) -> Option<Self::ObjectType> {
        match self {
            Self::MoveObject(o) => Some(o),
            Self::Package(_) => None,
        }
    }

    fn try_as_package(&self) -> Option<&Self::PackageType> {
        match self {
            Self::MoveObject(_) => None,
            Self::Package(p) => Some(p),
        }
    }

    fn type_(&self) -> Option<&StructTag> {
        match self {
            Self::MoveObject(o) => Some(&o.type_),
            Self::Package(_) => None,
        }
    }
}

impl Display for SuiParsedData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        match self {
            SuiParsedData::MoveObject(o) => {
                writeln!(writer, "{}: {}", "type".bold().bright_black(), o.type_)?;
                write!(writer, "{}", &o.fields)?;
            }
            SuiParsedData::Package(p) => {
                write!(
                    writer,
                    "{}: {:?}",
                    "Modules".bold().bright_black(),
                    p.disassembled.keys()
                )?;
            }
        }
        write!(f, "{}", writer)
    }
}

pub trait SuiMoveObject: Sized {
    fn type_(&self) -> &StructTag;
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename = "MoveObject", rename_all = "camelCase")]
pub struct SuiParsedMoveObject {
    #[serde(rename = "type")]
    // #[serde_as(as = "SuiStructTag")]
    #[serde_as(as = "DisplayFromStr")]
    pub type_: StructTag,
    pub has_public_transfer: bool,
    pub fields: SuiMoveStruct,
}

impl SuiMoveObject for SuiParsedMoveObject {
    fn type_(&self) -> &StructTag {
        &self.type_
    }
}

impl SuiParsedMoveObject {
    pub fn read_dynamic_field_value(&self, field_name: &str) -> Option<SuiMoveValue> {
        match &self.fields {
            SuiMoveStruct::WithFields(fields) => fields.get(field_name).cloned(),
            SuiMoveStruct::WithTypes { fields, .. } => fields.get(field_name).cloned(),
            _ => None,
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename = "RawMoveObject", rename_all = "camelCase")]
pub struct SuiRawMoveObject {
    #[serde(rename = "type")]
    // #[serde_as(as = "SuiStructTag")]
    #[serde_as(as = "DisplayFromStr")]
    pub type_: StructTag,
    pub has_public_transfer: bool,
    pub version: Version,
    #[serde_as(as = "Base64")]
    pub bcs_bytes: Vec<u8>,
}

impl SuiMoveObject for SuiRawMoveObject {
    fn type_(&self) -> &StructTag {
        &self.type_
    }
}

impl SuiRawMoveObject {
    pub fn deserialize<'a, T: Deserialize<'a>>(&'a self) -> Result<T, bcs::Error> {
        bcs::from_bytes(self.bcs_bytes.as_slice())
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename = "RawMovePackage", rename_all = "camelCase")]
pub struct SuiRawMovePackage {
    pub id: ObjectId,
    pub version: Version,
    #[serde_as(as = "BTreeMap<_, Base64>")]
    pub module_map: BTreeMap<String, Vec<u8>>,
    pub type_origin_table: Vec<TypeOrigin>,
    pub linkage_table: BTreeMap<ObjectId, UpgradeInfo>,
}

/// Errors for [`SuiPastObjectResponse`].
#[derive(thiserror::Error, Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Hash)]
pub enum SuiPastObjectResponseError {
    #[error("Could not find the referenced object {object_id:?} at version {version:?}.")]
    ObjectNotFound {
        object_id: ObjectId,
        version: Option<Version>,
    },

    #[error(
        "Could not find the referenced object {object_id:?} \
            as the asked version {asked_version:?} \
            is higher than the latest {latest_version:?}"
    )]
    ObjectSequenceNumberTooHigh {
        object_id: ObjectId,
        asked_version: Version,
        latest_version: Version,
    },

    #[error("Object deleted at reference {object_ref:?}.")]
    ObjectDeleted { object_ref: ObjectRef },
}

#[rustversion::attr(nightly, expect(clippy::large_enum_variant))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "status", content = "details", rename = "ObjectRead")]
pub enum SuiPastObjectResponse {
    /// The object exists and is found with this version
    VersionFound(SuiObjectData),
    /// The object does not exist
    ObjectNotExists(ObjectId),
    /// The object is found to be deleted with this version
    ObjectDeleted(SuiObjectRef),
    /// The object exists but not found with this version
    VersionNotFound(ObjectId, Version),
    /// The asked object version is higher than the latest
    VersionTooHigh {
        object_id: ObjectId,
        asked_version: Version,
        latest_version: Version,
    },
}

impl SuiPastObjectResponse {
    /// Returns a reference to the object if there is any, otherwise an Err
    pub fn object(&self) -> Result<&SuiObjectData, SuiPastObjectResponseError> {
        match &self {
            Self::ObjectDeleted(oref) => Err(SuiPastObjectResponseError::ObjectDeleted {
                object_ref: oref.to_object_ref(),
            }),
            Self::ObjectNotExists(id) => Err(SuiPastObjectResponseError::ObjectNotFound {
                object_id: *id,
                version: None,
            }),
            Self::VersionFound(o) => Ok(o),
            Self::VersionNotFound(id, seq_num) => Err(SuiPastObjectResponseError::ObjectNotFound {
                object_id: *id,
                version: Some(*seq_num),
            }),
            Self::VersionTooHigh {
                object_id,
                asked_version,
                latest_version,
            } => Err(SuiPastObjectResponseError::ObjectSequenceNumberTooHigh {
                object_id: *object_id,
                asked_version: *asked_version,
                latest_version: *latest_version,
            }),
        }
    }

    /// Returns the object value if there is any, otherwise an Err
    pub fn into_object(self) -> Result<SuiObjectData, SuiPastObjectResponseError> {
        match self {
            Self::ObjectDeleted(oref) => Err(SuiPastObjectResponseError::ObjectDeleted {
                object_ref: oref.to_object_ref(),
            }),
            Self::ObjectNotExists(id) => Err(SuiPastObjectResponseError::ObjectNotFound {
                object_id: id,
                version: None,
            }),
            Self::VersionFound(o) => Ok(o),
            Self::VersionNotFound(object_id, version) => {
                Err(SuiPastObjectResponseError::ObjectNotFound {
                    object_id,
                    version: Some(version),
                })
            }
            Self::VersionTooHigh {
                object_id,
                asked_version,
                latest_version,
            } => Err(SuiPastObjectResponseError::ObjectSequenceNumberTooHigh {
                object_id,
                asked_version,
                latest_version,
            }),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename = "MovePackage", rename_all = "camelCase")]
pub struct SuiMovePackage {
    pub disassembled: BTreeMap<String, Value>,
}

pub type QueryObjectsPage = Page<SuiObjectResponse, CheckpointedObjectId>;
pub type ObjectsPage = Page<SuiObjectResponse, ObjectId>;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointedObjectId {
    pub object_id: ObjectId,
    #[serde_as(as = "Option<BigInt<u64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_checkpoint: Option<Version>,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename = "GetPastObjectRequest", rename_all = "camelCase")]
pub struct SuiGetPastObjectRequest {
    /// the ID of the queried object
    pub object_id: ObjectId,
    /// the version of the queried object.
    #[serde_as(as = "BigInt<u64>")]
    pub version: Version,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SuiObjectDataFilter {
    MatchAll(Vec<SuiObjectDataFilter>),
    MatchAny(Vec<SuiObjectDataFilter>),
    MatchNone(Vec<SuiObjectDataFilter>),
    /// Query by type a specified Package.
    Package(ObjectId),
    /// Query by type a specified Move module.
    MoveModule {
        /// the Move package ID
        package: ObjectId,
        /// the module name
        #[serde_as(as = "DisplayFromStr")]
        module: Identifier,
    },
    /// Query by type
    // StructType(#[serde_as(as = "SuiStructTag")] StructTag),
    StructType(#[serde_as(as = "DisplayFromStr")] StructTag),
    AddressOwner(SuiAddress),
    ObjectOwner(ObjectId),
    ObjectId(ObjectId),
    // allow querying for multiple object ids
    ObjectIds(Vec<ObjectId>),
    Version(#[serde_as(as = "BigInt<u64>")] u64),
}

impl SuiObjectDataFilter {
    pub fn gas_coin() -> Self {
        Self::StructType(StructTag::gas_coin())
    }

    pub fn and(self, other: Self) -> Self {
        Self::MatchAll(vec![self, other])
    }
    pub fn or(self, other: Self) -> Self {
        Self::MatchAny(vec![self, other])
    }
    pub fn not(self, other: Self) -> Self {
        Self::MatchNone(vec![self, other])
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", rename = "ObjectResponseQuery", default)]
pub struct SuiObjectResponseQuery {
    /// If None, no filter will be applied
    pub filter: Option<SuiObjectDataFilter>,
    /// config which fields to include in the response, by default only digest is included
    pub options: Option<SuiObjectDataOptions>,
}

impl SuiObjectResponseQuery {
    pub fn new(filter: Option<SuiObjectDataFilter>, options: Option<SuiObjectDataOptions>) -> Self {
        Self { filter, options }
    }

    pub fn new_with_filter(filter: SuiObjectDataFilter) -> Self {
        Self {
            filter: Some(filter),
            options: None,
        }
    }

    pub fn new_with_options(options: SuiObjectDataOptions) -> Self {
        Self {
            filter: None,
            options: Some(options),
        }
    }
}
