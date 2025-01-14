// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
// TODO: replace this fully with `MovePackage` from `sui-sdk-types`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};
use sui_sdk_types::types::Version;

use crate::ObjectId;

#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MovePackage {
    pub id: ObjectId,
    /// Most move packages are uniquely identified by their ID (i.e. there is only one version per
    /// ID), but the version is still stored because one package may be an upgrade of another (at a
    /// different ID), in which case its version will be one greater than the version of the
    /// upgraded package.
    ///
    /// Framework packages are an exception to this rule -- all versions of the framework packages
    /// exist at the same ID, at increasing versions.
    ///
    /// In all cases, packages are referred to by move calls using just their ID, and they are
    /// always loaded at their latest version.
    pub version: Version,
    // TODO use session cache
    #[serde_as(as = "BTreeMap<_, Bytes>")]
    pub module_map: BTreeMap<String, Vec<u8>>,

    /// Maps struct/module to a package version where it was first defined, stored as a vector for
    /// simple serialization and deserialization.
    pub type_origin_table: Vec<TypeOrigin>,

    // For each dependency, maps original package ID to the info about the (upgraded) dependency
    // version that this package is using
    pub linkage_table: BTreeMap<ObjectId, UpgradeInfo>,
}

/// Identifies a struct and the module it was defined in
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, Hash)]
pub struct TypeOrigin {
    pub module_name: String,
    #[serde(alias = "struct_name")]
    pub datatype_name: String,
    pub package: ObjectId,
}

/// Upgraded package info for the linkage table
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct UpgradeInfo {
    /// ID of the upgraded packages
    pub upgraded_id: ObjectId,
    /// Version of the upgraded package
    pub upgraded_version: Version,
}
