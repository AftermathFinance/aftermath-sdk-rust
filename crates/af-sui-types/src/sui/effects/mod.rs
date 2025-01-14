// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use sui_sdk_types::{
    EpochId,
    ExecutionStatus,
    GasCostSummary,
    ObjectId,
    TransactionDigest,
    TransactionEventsDigest,
    Version,
};

use crate::{ObjectRef, Owner};

mod api;
mod v1;
mod v2;

// TODO: remove
// - TransactionEffects
// - everything under self::{v1, v2};
// in favor of the sui-sdk-types equivalents. Then implement TransactionEffectsAPI for
// sui_sdk_types::TransactionEffects for compatibility purposes

pub use self::api::{InputSharedObject, ObjectChange, TransactionEffectsAPI};
pub use self::v1::TransactionEffectsV1;
pub use self::v2::{
    EffectsObjectChange,
    IDOperation,
    ObjectIn,
    ObjectOut,
    TransactionEffectsV2,
    UnchangedSharedKind,
};

/// The response from processing a transaction or a certified transaction
#[enum_dispatch(TransactionEffectsAPI)]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum TransactionEffects {
    V1(TransactionEffectsV1),
    V2(TransactionEffectsV2),
}
