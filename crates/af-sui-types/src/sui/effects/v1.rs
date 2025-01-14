// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashSet};

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

use super::{
    IDOperation,
    InputSharedObject,
    ObjectChange,
    TransactionEffectsAPI,
    UnchangedSharedKind,
};
use crate::{ObjectRef, Owner};

/// The response from processing a transaction or a certified transaction
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct TransactionEffectsV1 {
    /// The status of the execution
    pub status: ExecutionStatus,
    /// The epoch when this transaction was executed.
    pub executed_epoch: EpochId,
    pub gas_used: GasCostSummary,
    /// The version that every modified (mutated or deleted) object had before it was modified by
    /// this transaction.
    pub modified_at_versions: Vec<(ObjectId, Version)>,
    /// The object references of the shared objects used in this transaction. Empty if no shared objects were used.
    pub shared_objects: Vec<ObjectRef>,
    /// The transaction digest
    pub transaction_digest: TransactionDigest,

    /// ObjectRef and owner of new objects created.
    pub created: Vec<(ObjectRef, Owner)>,
    /// ObjectRef and owner of mutated objects, including gas object.
    pub mutated: Vec<(ObjectRef, Owner)>,
    /// ObjectRef and owner of objects that are unwrapped in this transaction.
    /// Unwrapped objects are objects that were wrapped into other objects in the past,
    /// and just got extracted out.
    pub unwrapped: Vec<(ObjectRef, Owner)>,
    /// Object Refs of objects now deleted (the new refs).
    pub deleted: Vec<ObjectRef>,
    /// Object refs of objects previously wrapped in other objects but now deleted.
    pub unwrapped_then_deleted: Vec<ObjectRef>,
    /// Object refs of objects now wrapped in other objects.
    pub wrapped: Vec<ObjectRef>,
    /// The updated gas object reference. Have a dedicated field for convenient access.
    /// It's also included in mutated.
    pub gas_object: (ObjectRef, Owner),
    /// The digest of the events emitted during execution,
    /// can be None if the transaction does not emit any event.
    pub events_digest: Option<TransactionEventsDigest>,
    /// The set of transaction digests this transaction depends on.
    pub dependencies: Vec<TransactionDigest>,
}

impl TransactionEffectsAPI for TransactionEffectsV1 {
    fn status(&self) -> &ExecutionStatus {
        &self.status
    }

    fn into_status(self) -> ExecutionStatus {
        self.status
    }

    fn executed_epoch(&self) -> EpochId {
        self.executed_epoch
    }

    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)> {
        self.modified_at_versions
            .iter()
            // V1 transaction effects "modified_at_versions" includes unwrapped_then_deleted
            // objects, so in order to have parity with the V2 transaction effects semantics of
            // "modified_at_versions", filter out any objects that are unwrapped_then_deleted'ed
            .filter(|(object_id, _)| {
                !self
                    .unwrapped_then_deleted
                    .iter()
                    .any(|(deleted_id, _, _)| deleted_id == object_id)
            })
            .cloned()
            .collect()
    }

    fn lamport_version(&self) -> Version {
        self.modified_at_versions
            .iter()
            .map(|(_, v)| *v)
            .fold(0, std::cmp::max)
            + 1
    }

    fn old_object_metadata(&self) -> Vec<(ObjectRef, Owner)> {
        unimplemented!("Only supposed by v2 and above");
    }

    fn sequenced_input_shared_objects(&self) -> Vec<InputSharedObject> {
        let modified: HashSet<_> = self.modified_at_versions.iter().map(|(r, _)| r).collect();
        self.shared_objects
            .iter()
            .map(|r| {
                if modified.contains(&r.0) {
                    InputSharedObject::Mutate(*r)
                } else {
                    InputSharedObject::ReadOnly(*r)
                }
            })
            .collect()
    }

    fn created(&self) -> Vec<(ObjectRef, Owner)> {
        self.created.clone()
    }

    fn mutated(&self) -> Vec<(ObjectRef, Owner)> {
        self.mutated.clone()
    }

    fn unwrapped(&self) -> Vec<(ObjectRef, Owner)> {
        self.unwrapped.clone()
    }

    fn deleted(&self) -> Vec<ObjectRef> {
        self.deleted.clone()
    }

    fn unwrapped_then_deleted(&self) -> Vec<ObjectRef> {
        self.unwrapped_then_deleted.clone()
    }

    fn wrapped(&self) -> Vec<ObjectRef> {
        self.wrapped.clone()
    }

    fn object_changes(&self) -> Vec<ObjectChange> {
        let modified_at: BTreeMap<_, _> = self.modified_at_versions.iter().copied().collect();

        let created = self.created.iter().map(|((id, v, d), _)| ObjectChange {
            id: *id,
            input_version: None,
            input_digest: None,
            output_version: Some(*v),
            output_digest: Some(*d),
            id_operation: IDOperation::Created,
        });

        let mutated = self.mutated.iter().map(|((id, v, d), _)| ObjectChange {
            id: *id,
            input_version: modified_at.get(id).copied(),
            input_digest: None,
            output_version: Some(*v),
            output_digest: Some(*d),
            id_operation: IDOperation::None,
        });

        let unwrapped = self.unwrapped.iter().map(|((id, v, d), _)| ObjectChange {
            id: *id,
            input_version: None,
            input_digest: None,
            output_version: Some(*v),
            output_digest: Some(*d),
            id_operation: IDOperation::None,
        });

        let deleted = self.deleted.iter().map(|(id, _, _)| ObjectChange {
            id: *id,
            input_version: modified_at.get(id).copied(),
            input_digest: None,
            output_version: None,
            output_digest: None,
            id_operation: IDOperation::Deleted,
        });

        let unwrapped_then_deleted =
            self.unwrapped_then_deleted
                .iter()
                .map(|(id, _, _)| ObjectChange {
                    id: *id,
                    input_version: None,
                    input_digest: None,
                    output_version: None,
                    output_digest: None,
                    id_operation: IDOperation::Deleted,
                });

        let wrapped = self.wrapped.iter().map(|(id, _, _)| ObjectChange {
            id: *id,
            input_version: modified_at.get(id).copied(),
            input_digest: None,
            output_version: None,
            output_digest: None,
            id_operation: IDOperation::None,
        });

        created
            .chain(mutated)
            .chain(unwrapped)
            .chain(deleted)
            .chain(unwrapped_then_deleted)
            .chain(wrapped)
            .collect()
    }

    fn gas_object(&self) -> Option<(ObjectRef, Owner)> {
        Some(self.gas_object.clone())
    }
    fn events_digest(&self) -> Option<&TransactionEventsDigest> {
        self.events_digest.as_ref()
    }

    fn dependencies(&self) -> &[TransactionDigest] {
        &self.dependencies
    }

    fn transaction_digest(&self) -> &TransactionDigest {
        &self.transaction_digest
    }

    fn gas_cost_summary(&self) -> &GasCostSummary {
        &self.gas_used
    }

    fn unchanged_shared_objects(&self) -> Vec<(ObjectId, UnchangedSharedKind)> {
        self.sequenced_input_shared_objects()
            .iter()
            .filter_map(|o| match o {
                // In effects v1, the only unchanged shared objects are read-only shared objects.
                InputSharedObject::ReadOnly(oref) => {
                    Some((oref.0, UnchangedSharedKind::ReadOnlyRoot((oref.1, oref.2))))
                }
                _ => None,
            })
            .collect()
    }
}
