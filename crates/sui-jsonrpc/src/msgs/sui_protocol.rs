// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use af_sui_types::ProtocolVersion;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, IfIsHumanReadable, serde_as};

use crate::serde::BigInt;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", rename = "ProtocolConfig")]
pub struct ProtocolConfigResponse {
    #[serde_as(as = "IfIsHumanReadable<DisplayFromStr, _>")]
    pub min_supported_protocol_version: ProtocolVersion,
    #[serde_as(as = "IfIsHumanReadable<DisplayFromStr, _>")]
    pub max_supported_protocol_version: ProtocolVersion,
    #[serde_as(as = "IfIsHumanReadable<DisplayFromStr, _>")]
    pub protocol_version: ProtocolVersion,
    pub feature_flags: BTreeMap<String, bool>,
    pub attributes: BTreeMap<String, Option<SuiProtocolConfigValue>>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", rename = "ProtocolConfigValue")]
pub enum SuiProtocolConfigValue {
    U16(#[serde_as(as = "BigInt<u16>")] u16),
    U32(#[serde_as(as = "BigInt<u32>")] u32),
    U64(#[serde_as(as = "BigInt<u64>")] u64),
    F64(#[serde_as(as = "DisplayFromStr")] f64),
    Bool(#[serde_as(as = "DisplayFromStr")] bool),
}
