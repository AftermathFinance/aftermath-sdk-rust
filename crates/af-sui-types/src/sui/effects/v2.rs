// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use sui_sdk_types::types::{
    EffectsAuxiliaryDataDigest,
    EpochId,
    ObjectDigest,
    TransactionDigest,
    TransactionEventsDigest,
    Version,
};

use super::{InputSharedObject, ObjectChange, TransactionEffectsAPI};
use crate::{
    ExecutionStatus,
    GasCostSummary,
    ObjectId,
    ObjectRef,
    Owner,
    OBJECT_DIGEST_DELETED,
    OBJECT_DIGEST_WRAPPED,
};

type VersionDigest = (Version, ObjectDigest);

/// The response from processing a transaction or a certified transaction
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct TransactionEffectsV2 {
    /// The status of the execution
    pub status: ExecutionStatus,
    /// The epoch when this transaction was executed.
    pub executed_epoch: EpochId,
    pub gas_used: GasCostSummary,
    /// The transaction digest
    pub transaction_digest: TransactionDigest,
    /// The updated gas object reference, as an index into the `changed_objects` vector.
    /// Having a dedicated field for convenient access.
    /// System transaction that don't require gas will leave this as None.
    pub gas_object_index: Option<u32>,
    /// The digest of the events emitted during execution,
    /// can be None if the transaction does not emit any event.
    pub events_digest: Option<TransactionEventsDigest>,
    /// The set of transaction digests this transaction depends on.
    pub dependencies: Vec<TransactionDigest>,

    /// The version number of all the written Move objects by this transaction.
    pub lamport_version: Version,
    /// Objects whose state are changed in the object store.
    pub changed_objects: Vec<(ObjectId, EffectsObjectChange)>,
    /// Shared objects that are not mutated in this transaction. Unlike owned objects,
    /// read-only shared objects' version are not committed in the transaction,
    /// and in order for a node to catch up and execute it without consensus sequencing,
    /// the version needs to be committed in the effects.
    pub unchanged_shared_objects: Vec<(ObjectId, UnchangedSharedKind)>,
    /// Auxiliary data that are not protocol-critical, generated as part of the effects but are stored separately.
    /// Storing it separately allows us to avoid bloating the effects with data that are not critical.
    /// It also provides more flexibility on the format and type of the data.
    pub aux_data_digest: Option<EffectsAuxiliaryDataDigest>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct EffectsObjectChange {
    // input_state and output_state are the core fields that's required by
    // the protocol as it tells how an object changes on-chain.
    /// State of the object in the store prior to this transaction.
    pub input_state: ObjectIn,
    /// State of the object in the store after this transaction.
    pub output_state: ObjectOut,

    /// Whether this object ID is created or deleted in this transaction.
    /// This information isn't required by the protocol but is useful for providing more detailed
    /// semantics on object changes.
    pub id_operation: IDOperation,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum UnchangedSharedKind {
    /// Read-only shared objects from the input. We don't really need ObjectDigest
    /// for protocol correctness, but it will make it easier to verify untrusted read.
    ReadOnlyRoot(VersionDigest),
    /// Deleted shared objects that appear mutably/owned in the input.
    MutateDeleted(Version),
    /// Deleted shared objects that appear as read-only in the input.
    ReadDeleted(Version),
    /// Shared objects in cancelled transaction. The sequence number embed cancellation reason.
    Cancelled(Version),
    /// Read of a per-epoch config object that should remain the same during an epoch.
    PerEpochConfig,
}

/// If an object exists (at root-level) in the store prior to this transaction,
/// it should be Exist, otherwise it's NonExist, e.g. wrapped objects should be
/// NonExist.
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum ObjectIn {
    NotExist,
    /// The old version, digest and owner.
    Exist((VersionDigest, Owner)),
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum ObjectOut {
    /// Same definition as in ObjectIn.
    NotExist,
    /// Any written object, including all of mutated, created, unwrapped today.
    ObjectWrite((ObjectDigest, Owner)),
    /// Packages writes need to be tracked separately with version because
    /// we don't use lamport version for package publish and upgrades.
    PackageWrite(VersionDigest),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum IDOperation {
    None,
    Created,
    Deleted,
}

impl TransactionEffectsAPI for TransactionEffectsV2 {
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
        self.changed_objects
            .iter()
            .filter_map(|(id, change)| {
                if let ObjectIn::Exist(((version, _digest), _owner)) = &change.input_state {
                    Some((*id, *version))
                } else {
                    None
                }
            })
            .collect()
    }

    fn lamport_version(&self) -> Version {
        self.lamport_version
    }

    fn old_object_metadata(&self) -> Vec<(ObjectRef, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(|(id, change)| {
                if let ObjectIn::Exist(((version, digest), owner)) = &change.input_state {
                    Some(((*id, *version, *digest), owner.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn sequenced_input_shared_objects(&self) -> Vec<InputSharedObject> {
        self.changed_objects
            .iter()
            .filter_map(|(id, change)| match &change.input_state {
                ObjectIn::Exist(((version, digest), Owner::Shared { .. })) => {
                    Some(InputSharedObject::Mutate((*id, *version, *digest)))
                }
                _ => None,
            })
            .chain(
                self.unchanged_shared_objects
                    .iter()
                    .filter_map(|(id, change_kind)| match change_kind {
                        UnchangedSharedKind::ReadOnlyRoot((version, digest)) => {
                            Some(InputSharedObject::ReadOnly((*id, *version, *digest)))
                        }
                        UnchangedSharedKind::MutateDeleted(seqno) => {
                            Some(InputSharedObject::MutateDeleted(*id, *seqno))
                        }
                        UnchangedSharedKind::ReadDeleted(seqno) => {
                            Some(InputSharedObject::ReadDeleted(*id, *seqno))
                        }
                        UnchangedSharedKind::Cancelled(seqno) => {
                            Some(InputSharedObject::Cancelled(*id, *seqno))
                        }
                        // We can not expose the per epoch config object as input shared object,
                        // since it does not require sequencing, and hence shall not be considered
                        // as a normal input shared object.
                        UnchangedSharedKind::PerEpochConfig => None,
                    }),
            )
            .collect()
    }

    fn created(&self) -> Vec<(ObjectRef, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(|(id, change)| {
                match (
                    &change.input_state,
                    &change.output_state,
                    &change.id_operation,
                ) {
                    (
                        ObjectIn::NotExist,
                        ObjectOut::ObjectWrite((digest, owner)),
                        IDOperation::Created,
                    ) => Some(((*id, self.lamport_version, *digest), owner.clone())),
                    (
                        ObjectIn::NotExist,
                        ObjectOut::PackageWrite((version, digest)),
                        IDOperation::Created,
                    ) => Some(((*id, *version, *digest), Owner::Immutable)),
                    _ => None,
                }
            })
            .collect()
    }

    fn mutated(&self) -> Vec<(ObjectRef, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(
                |(id, change)| match (&change.input_state, &change.output_state) {
                    (ObjectIn::Exist(_), ObjectOut::ObjectWrite((digest, owner))) => {
                        Some(((*id, self.lamport_version, *digest), owner.clone()))
                    }
                    (ObjectIn::Exist(_), ObjectOut::PackageWrite((version, digest))) => {
                        Some(((*id, *version, *digest), Owner::Immutable))
                    }
                    _ => None,
                },
            )
            .collect()
    }

    fn unwrapped(&self) -> Vec<(ObjectRef, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(|(id, change)| {
                match (
                    &change.input_state,
                    &change.output_state,
                    &change.id_operation,
                ) {
                    (
                        ObjectIn::NotExist,
                        ObjectOut::ObjectWrite((digest, owner)),
                        IDOperation::None,
                    ) => Some(((*id, self.lamport_version, *digest), owner.clone())),
                    _ => None,
                }
            })
            .collect()
    }

    fn deleted(&self) -> Vec<ObjectRef> {
        self.changed_objects
            .iter()
            .filter_map(|(id, change)| {
                match (
                    &change.input_state,
                    &change.output_state,
                    &change.id_operation,
                ) {
                    (ObjectIn::Exist(_), ObjectOut::NotExist, IDOperation::Deleted) => {
                        Some((*id, self.lamport_version, OBJECT_DIGEST_DELETED))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    fn unwrapped_then_deleted(&self) -> Vec<ObjectRef> {
        self.changed_objects
            .iter()
            .filter_map(|(id, change)| {
                match (
                    &change.input_state,
                    &change.output_state,
                    &change.id_operation,
                ) {
                    (ObjectIn::NotExist, ObjectOut::NotExist, IDOperation::Deleted) => {
                        Some((*id, self.lamport_version, OBJECT_DIGEST_DELETED))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    fn wrapped(&self) -> Vec<ObjectRef> {
        self.changed_objects
            .iter()
            .filter_map(|(id, change)| {
                match (
                    &change.input_state,
                    &change.output_state,
                    &change.id_operation,
                ) {
                    (ObjectIn::Exist(_), ObjectOut::NotExist, IDOperation::None) => {
                        Some((*id, self.lamport_version, OBJECT_DIGEST_WRAPPED))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    fn object_changes(&self) -> Vec<ObjectChange> {
        self.changed_objects
            .iter()
            .map(|(id, change)| {
                let input_version_digest = match &change.input_state {
                    ObjectIn::NotExist => None,
                    ObjectIn::Exist((vd, _)) => Some(*vd),
                };

                let output_version_digest = match &change.output_state {
                    ObjectOut::NotExist => None,
                    ObjectOut::ObjectWrite((d, _)) => Some((self.lamport_version, *d)),
                    ObjectOut::PackageWrite(vd) => Some(*vd),
                };

                ObjectChange {
                    id: *id,

                    input_version: input_version_digest.map(|k| k.0),
                    input_digest: input_version_digest.map(|k| k.1),

                    output_version: output_version_digest.map(|k| k.0),
                    output_digest: output_version_digest.map(|k| k.1),

                    id_operation: change.id_operation,
                }
            })
            .collect()
    }

    fn gas_object(&self) -> Option<(ObjectRef, Owner)> {
        self.gas_object_index.map(|gas_object_index| {
            let entry = &self.changed_objects[gas_object_index as usize];
            match entry.1.output_state {
                ObjectOut::ObjectWrite((digest, ref owner)) => {
                    ((entry.0, self.lamport_version, digest), owner.clone())
                }
                _ => panic!("Gas object must be an ObjectWrite in changed_objects"),
            }
        })
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
        self.unchanged_shared_objects.clone()
    }
}
