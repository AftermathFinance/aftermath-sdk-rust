// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use af_sui_types::{
    Address as SuiAddress,
    ObjectDigest,
    ObjectId,
    ObjectRef,
    Owner,
    StructTag,
    OBJECT_DIGEST_DELETED,
    OBJECT_DIGEST_WRAPPED,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use sui_sdk_types::types::Version;

use crate::serde::BigInt;

/// ObjectChange are derived from the object mutations in the TransactionEffect to provide richer object information.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ObjectChange {
    /// Module published
    #[serde(rename_all = "camelCase")]
    Published {
        package_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        version: Version,
        digest: ObjectDigest,
        modules: Vec<String>,
    },
    /// Transfer objects to new address / wrap in another object
    #[serde(rename_all = "camelCase")]
    Transferred {
        sender: SuiAddress,
        recipient: Owner,
        // #[serde_as(as = "SuiStructTag")]
        #[serde_as(as = "DisplayFromStr")]
        object_type: StructTag,
        object_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        version: Version,
        digest: ObjectDigest,
    },
    /// Object mutated.
    #[serde(rename_all = "camelCase")]
    Mutated {
        sender: SuiAddress,
        owner: Owner,
        // #[serde_as(as = "SuiStructTag")]
        #[serde_as(as = "DisplayFromStr")]
        object_type: StructTag,
        object_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        version: Version,
        #[serde_as(as = "BigInt<u64>")]
        previous_version: Version,
        digest: ObjectDigest,
    },
    /// Delete object
    #[serde(rename_all = "camelCase")]
    Deleted {
        sender: SuiAddress,
        // #[serde_as(as = "SuiStructTag")]
        #[serde_as(as = "DisplayFromStr")]
        object_type: StructTag,
        object_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        version: Version,
    },
    /// Wrapped object
    #[serde(rename_all = "camelCase")]
    Wrapped {
        sender: SuiAddress,
        // #[serde_as(as = "SuiStructTag")]
        #[serde_as(as = "DisplayFromStr")]
        object_type: StructTag,
        object_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        version: Version,
    },
    /// New object creation
    #[serde(rename_all = "camelCase")]
    Created {
        sender: SuiAddress,
        owner: Owner,
        // #[serde_as(as = "SuiStructTag")]
        #[serde_as(as = "DisplayFromStr")]
        object_type: StructTag,
        object_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        version: Version,
        digest: ObjectDigest,
    },
}

impl ObjectChange {
    pub fn object_id(&self) -> ObjectId {
        match self {
            ObjectChange::Published { package_id, .. } => *package_id,
            ObjectChange::Transferred { object_id, .. }
            | ObjectChange::Mutated { object_id, .. }
            | ObjectChange::Deleted { object_id, .. }
            | ObjectChange::Wrapped { object_id, .. }
            | ObjectChange::Created { object_id, .. } => *object_id,
        }
    }

    pub fn object_ref(&self) -> ObjectRef {
        match self {
            ObjectChange::Published {
                package_id,
                version,
                digest,
                ..
            } => (*package_id, *version, *digest),
            ObjectChange::Transferred {
                object_id,
                version,
                digest,
                ..
            }
            | ObjectChange::Mutated {
                object_id,
                version,
                digest,
                ..
            }
            | ObjectChange::Created {
                object_id,
                version,
                digest,
                ..
            } => (*object_id, *version, *digest),
            ObjectChange::Deleted {
                object_id, version, ..
            } => (*object_id, *version, OBJECT_DIGEST_DELETED),
            ObjectChange::Wrapped {
                object_id, version, ..
            } => (*object_id, *version, OBJECT_DIGEST_WRAPPED),
        }
    }
}

impl Display for ObjectChange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ObjectChange::Published {
                package_id,
                version,
                digest,
                modules,
            } => {
                write!(
                    f,
                    " ┌──\n │ PackageID: {} \n │ Version: {} \n │ Digest: {}\n │ Modules: {}\n └──",
                    package_id,
                    version,
                    digest,
                    modules.join(", ")
                )
            }
            ObjectChange::Transferred {
                sender,
                recipient,
                object_type,
                object_id,
                version,
                digest,
            } => {
                write!(
                    f,
                    " ┌──\n │ ObjectId: {}\n │ Sender: {} \n │ Recipient: {}\n │ ObjectType: {} \n │ Version: {}\n │ Digest: {}\n └──",
                    object_id, sender, recipient, object_type, version, digest
                )
            }
            ObjectChange::Mutated {
                sender,
                owner,
                object_type,
                object_id,
                version,
                previous_version: _,
                digest,
            } => {
                write!(
                    f,
                    " ┌──\n │ ObjectId: {}\n │ Sender: {} \n │ Owner: {}\n │ ObjectType: {} \n │ Version: {}\n │ Digest: {}\n └──",
                    object_id, sender, owner, object_type, version, digest
                )
            }
            ObjectChange::Deleted {
                sender,
                object_type,
                object_id,
                version,
            } => {
                write!(
                    f,
                    " ┌──\n │ ObjectId: {}\n │ Sender: {} \n │ ObjectType: {} \n │ Version: {}\n └──",
                    object_id, sender, object_type, version
                )
            }
            ObjectChange::Wrapped {
                sender,
                object_type,
                object_id,
                version,
            } => {
                write!(
                    f,
                    " ┌──\n │ ObjectId: {}\n │ Sender: {} \n │ ObjectType: {} \n │ Version: {}\n └──",
                    object_id, sender, object_type, version
                )
            }
            ObjectChange::Created {
                sender,
                owner,
                object_type,
                object_id,
                version,
                digest,
            } => {
                write!(
                    f,
                    " ┌──\n │ ObjectId: {}\n │ Sender: {} \n │ Owner: {}\n │ ObjectType: {} \n │ Version: {}\n │ Digest: {}\n └──",
                    object_id, sender, owner, object_type, version, digest
                )
            }
        }
    }
}
