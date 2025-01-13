// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sui_sdk_types::{CheckpointContents, SignedCheckpointSummary, Transaction, TransactionEvents};

use crate::sui::transaction::_serde::SignedTransactionWithIntentMessage;
use crate::{Object, ObjectRef, SignedTransaction, TransactionEffects, TransactionEffectsAPI as _};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointData {
    pub checkpoint_summary: SignedCheckpointSummary,
    pub checkpoint_contents: CheckpointContents,
    pub transactions: Vec<CheckpointTransaction>,
}

impl CheckpointData {
    /// The latest versions of the output objects that still exist at the end of the checkpoint
    pub fn latest_live_output_objects(&self) -> Vec<&Object> {
        live_tx_output_objects(&self.transactions).collect()
    }

    /// The object refs that are eventually deleted or wrapped in the current checkpoint
    pub fn eventually_removed_object_refs_post_version(&self) -> Vec<ObjectRef> {
        let mut eventually_removed_object_refs = BTreeMap::new();
        for tx in self.transactions.iter() {
            for obj_ref in tx.effects.removed_object_refs_post_version() {
                eventually_removed_object_refs.insert(obj_ref.0, obj_ref);
            }
            for obj in tx.output_objects.iter() {
                eventually_removed_object_refs.remove(&(obj.id()));
            }
        }
        eventually_removed_object_refs.into_values().collect()
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointTransaction {
    /// The input Transaction
    #[serde_as(as = "SignedTransactionWithIntentMessage")]
    pub transaction: SignedTransaction,
    /// The effects produced by executing this transaction
    pub effects: TransactionEffects,
    /// The events, if any, emitted by this transaction during execution
    pub events: Option<TransactionEvents>,
    /// The state of all inputs to this transaction as they were prior to execution.
    pub input_objects: Vec<Object>,
    /// The state of all output objects created or mutated or unwrapped by this transaction.
    pub output_objects: Vec<Object>,
}

impl CheckpointTransaction {
    /// The unsigned transaction payload.
    pub const fn transaction_data(&self) -> &Transaction {
        &self.transaction.transaction
    }
}

/// The latest versions of the output objects that still exist after a sequence of transactions.
fn live_tx_output_objects<'a>(
    transactions: impl IntoIterator<Item = &'a CheckpointTransaction>,
) -> impl Iterator<Item = &'a Object> {
    let mut latest_live_objects = BTreeMap::new();
    for tx in transactions.into_iter() {
        for obj in tx.output_objects.iter() {
            latest_live_objects.insert(obj.id(), obj);
        }
        for (obj_id, _, _) in tx.effects.removed_object_refs_post_version() {
            latest_live_objects.remove(&obj_id);
        }
    }
    latest_live_objects.into_values()
}
