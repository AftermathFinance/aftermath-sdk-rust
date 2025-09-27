// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Common interface for all transaction effect versions.
use sui_sdk_types::{
    EpochId,
    ExecutionStatus,
    GasCostSummary,
    IdOperation,
    ObjectDigest,
    ObjectId,
    ObjectReference as ObjectRef,
    Owner,
    TransactionDigest,
    TransactionEventsDigest,
    UnchangedSharedKind,
    Version,
};

/// Common interface for all transaction effect versions.
///
/// This trait is inherited from Sui's monorepo and is here for legacy reasons.
pub trait TransactionEffectsAPI {
    fn status(&self) -> &ExecutionStatus;

    fn into_status(self) -> ExecutionStatus;

    fn executed_epoch(&self) -> EpochId;

    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)>;

    /// The version assigned to all output objects (apart from packages).
    fn lamport_version(&self) -> Version;

    /// Metadata of objects prior to modification. This includes any object that exists in the
    /// store prior to this transaction and is modified in this transaction.
    /// It includes objects that are mutated, wrapped and deleted.
    /// This API is only available on effects v2 and above.
    fn old_object_metadata(&self) -> Vec<(ObjectRef, Owner)>;

    /// Returns the list of sequenced shared objects used in the input.
    /// This is needed in effects because in transaction we only have object ID
    /// for shared objects. Their version and digest can only be figured out after sequencing.
    /// Also provides the use kind to indicate whether the object was mutated or read-only.
    /// It does not include per epoch config objects since they do not require sequencing.
    fn sequenced_input_shared_objects(&self) -> Vec<InputSharedObject>;

    fn created(&self) -> Vec<(ObjectRef, Owner)>;

    fn mutated(&self) -> Vec<(ObjectRef, Owner)>;

    /// All objects references that are inaccessible after this transaction.
    ///
    /// The union of all deleted, wrapped or unwrapped-then-deleted objects.
    fn removed_object_refs_post_version(&self) -> Box<dyn Iterator<Item = ObjectRef>> {
        let deleted = self.deleted().into_iter();
        let wrapped = self.wrapped().into_iter();
        let unwrapped_then_deleted = self.unwrapped_then_deleted().into_iter();
        Box::new(deleted.chain(wrapped).chain(unwrapped_then_deleted))
    }

    fn unwrapped(&self) -> Vec<(ObjectRef, Owner)>;

    fn deleted(&self) -> Vec<ObjectRef>;

    fn unwrapped_then_deleted(&self) -> Vec<ObjectRef>;

    fn wrapped(&self) -> Vec<ObjectRef>;

    fn object_changes(&self) -> Vec<ObjectChange>;

    // Returns `None` when the gas object is not available (i.e. system transaction).
    fn gas_object(&self) -> Option<(ObjectRef, Owner)>;

    fn events_digest(&self) -> Option<&TransactionEventsDigest>;

    fn dependencies(&self) -> &[TransactionDigest];

    fn transaction_digest(&self) -> &TransactionDigest;

    fn gas_cost_summary(&self) -> &GasCostSummary;

    fn deleted_mutably_accessed_shared_objects(&self) -> Vec<ObjectId> {
        self.sequenced_input_shared_objects()
            .into_iter()
            .filter_map(|kind| match kind {
                InputSharedObject::MutateDeleted(id, _) => Some(id),
                InputSharedObject::Mutate(..)
                | InputSharedObject::ReadOnly(..)
                | InputSharedObject::ReadDeleted(..)
                | InputSharedObject::Canceled(..) => None,
            })
            .collect()
    }

    /// Returns all root shared objects (i.e. not child object) that are read-only in the transaction.
    fn unchanged_shared_objects(&self) -> Vec<(ObjectId, UnchangedSharedKind)>;
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum InputSharedObject {
    Mutate(ObjectRef),
    ReadOnly(ObjectRef),
    ReadDeleted(ObjectId, Version),
    MutateDeleted(ObjectId, Version),
    Canceled(ObjectId, Version),
}

impl InputSharedObject {
    pub fn id_and_version(&self) -> (ObjectId, Version) {
        let oref = self.object_ref();
        (*oref.object_id(), oref.version())
    }

    pub fn object_ref(&self) -> ObjectRef {
        match self {
            Self::Mutate(oref) | Self::ReadOnly(oref) => oref.clone(),
            Self::ReadDeleted(id, version) | Self::MutateDeleted(id, version) => {
                ObjectRef::new(*id, *version, crate::OBJECT_DIGEST_DELETED)
            }
            Self::Canceled(id, version) => {
                ObjectRef::new(*id, *version, crate::OBJECT_DIGEST_CANCELLED)
            }
        }
    }
}

#[derive(Clone)]
pub struct ObjectChange {
    pub id: ObjectId,
    pub input_version: Option<Version>,
    pub input_digest: Option<ObjectDigest>,
    pub output_version: Option<Version>,
    pub output_digest: Option<ObjectDigest>,
    pub id_operation: IdOperation,
}
