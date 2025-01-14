// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use af_sui_types::{EpochId, ObjectDigest, ObjectId, ObjectRef, TransactionDigest};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, IfIsHumanReadable};
use sui_sdk_types::types::Version;

use super::Page;
use crate::serde::BigInt;

pub type CoinPage = Page<Coin, ObjectId>;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin_type: String,
    pub coin_object_count: usize,
    #[serde_as(as = "BigInt<u128>")]
    pub total_balance: u128,
    // TODO: This should be removed
    #[serde_as(as = "HashMap<BigInt<u64>, BigInt<u128>>")]
    pub locked_balance: HashMap<EpochId, u128>,
}

impl Balance {
    pub fn zero(coin_type: String) -> Self {
        Self {
            coin_type,
            coin_object_count: 0,
            total_balance: 0,
            locked_balance: HashMap::new(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Coin {
    pub coin_type: String,
    pub coin_object_id: ObjectId,
    #[serde_as(as = "BigInt<u64>")]
    pub version: Version,
    pub digest: ObjectDigest,
    #[serde_as(as = "BigInt<u64>")]
    pub balance: u64,
    pub previous_transaction: TransactionDigest,
}

impl Coin {
    pub fn object_ref(&self) -> ObjectRef {
        (self.coin_object_id, self.version, self.digest)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoinMetadata {
    /// Number of decimal places the coin uses.
    pub decimals: u8,
    /// Name for the token
    pub name: String,
    /// Symbol for the token
    pub symbol: String,
    /// Description of the token
    pub description: String,
    /// URL for the token logo
    pub icon_url: Option<String>,
    /// Object id for the CoinMetadata object
    pub id: Option<ObjectId>,
}

/// Originally from `sui_types::balance`.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Supply {
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub value: u64,
}
