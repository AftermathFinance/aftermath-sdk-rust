// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use std::borrow::Borrow;
use std::fmt::Display;

use af_sui_types::{
    IdentStr,
    ObjectDigest,
    ObjectId,
    StructTag,
    TypeTag,
    DYNAMIC_FIELD_FIELD_STRUCT_NAME,
    DYNAMIC_FIELD_MODULE_NAME,
    DYNAMIC_OBJECT_FIELD_MODULE_NAME,
    DYNAMIC_OBJECT_FIELD_WRAPPER_STRUCT_NAME,
    SUI_FRAMEWORK_ADDRESS,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, IfIsHumanReadable};
use sui_sdk_types::types::Version;

use super::Page;
use crate::serde::Base64orBase58;

pub type DynamicFieldPage = Page<DynamicFieldInfo, ObjectId>;

/// Originally `sui_types::dynamic_field::DynamicFieldName`.
#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DynamicFieldName {
    #[serde_as(as = "IfIsHumanReadable<DisplayFromStr, _>")]
    pub type_: TypeTag,
    #[serde_as(as = "IfIsHumanReadable<_, DisplayFromStr>")]
    pub value: serde_json::Value,
}

impl Display for DynamicFieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.type_, self.value)
    }
}

/// Originally `sui_types::dynamic_field::DynamicFieldInfo`.
#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DynamicFieldInfo {
    pub name: DynamicFieldName,
    #[serde_as(as = "IfIsHumanReadable<Base64orBase58, _>")]
    pub bcs_name: Vec<u8>,
    pub type_: DynamicFieldType,
    pub object_type: String,
    pub object_id: ObjectId,
    pub version: Version,
    pub digest: ObjectDigest,
}

impl DynamicFieldInfo {
    pub fn is_dynamic_field(tag: &StructTag) -> bool {
        tag.address == SUI_FRAMEWORK_ADDRESS
            && Borrow::<IdentStr>::borrow(&tag.module) == DYNAMIC_FIELD_MODULE_NAME
            && Borrow::<IdentStr>::borrow(&tag.name) == DYNAMIC_FIELD_FIELD_STRUCT_NAME
    }

    pub fn is_dynamic_object_field_wrapper(tag: &StructTag) -> bool {
        tag.address == SUI_FRAMEWORK_ADDRESS
            && Borrow::<IdentStr>::borrow(&tag.module) == DYNAMIC_OBJECT_FIELD_MODULE_NAME
            && Borrow::<IdentStr>::borrow(&tag.name) == DYNAMIC_OBJECT_FIELD_WRAPPER_STRUCT_NAME
    }

    pub fn dynamic_field_type(key: TypeTag, value: TypeTag) -> StructTag {
        StructTag {
            address: SUI_FRAMEWORK_ADDRESS,
            name: DYNAMIC_FIELD_FIELD_STRUCT_NAME.to_owned(),
            module: DYNAMIC_FIELD_MODULE_NAME.to_owned(),
            type_params: vec![key, value],
        }
    }

    pub fn dynamic_object_field_wrapper(key: TypeTag) -> StructTag {
        StructTag {
            address: SUI_FRAMEWORK_ADDRESS,
            module: DYNAMIC_OBJECT_FIELD_MODULE_NAME.to_owned(),
            name: DYNAMIC_OBJECT_FIELD_WRAPPER_STRUCT_NAME.to_owned(),
            type_params: vec![key],
        }
    }
}

/// Originally `sui_types::dynamic_field::DynamicFieldType`.
#[derive(Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum DynamicFieldType {
    #[serde(rename_all = "camelCase")]
    DynamicField,
    DynamicObject,
}

impl Display for DynamicFieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamicFieldType::DynamicField => write!(f, "DynamicField"),
            DynamicFieldType::DynamicObject => write!(f, "DynamicObject"),
        }
    }
}
