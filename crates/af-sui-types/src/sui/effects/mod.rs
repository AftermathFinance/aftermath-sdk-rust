// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashSet};

use sui_sdk_types::{
    EpochId,
    ExecutionStatus,
    GasCostSummary,
    IdOperation,
    ObjectId,
    ObjectIn,
    ObjectOut,
    ObjectReference,
    ObjectReferenceWithOwner,
    TransactionDigest,
    TransactionEffects,
    TransactionEffectsV1,
    TransactionEffectsV2,
    TransactionEventsDigest,
    UnchangedSharedKind,
    Version,
};

use crate::{ObjectRef, Owner, OBJECT_DIGEST_DELETED, OBJECT_DIGEST_WRAPPED};

mod api;

pub use self::api::{InputSharedObject, ObjectChange, TransactionEffectsAPI};

macro_rules! dispatch {
    ($self:ident, $method:ident) => {
        match $self {
            Self::V1(e) => e.$method(),
            Self::V2(e) => e.$method(),
        }
    };
}

impl TransactionEffectsAPI for TransactionEffects {
    fn status(&self) -> &ExecutionStatus {
        dispatch!(self, status)
    }

    fn into_status(self) -> ExecutionStatus {
        dispatch!(self, into_status)
    }

    fn executed_epoch(&self) -> EpochId {
        dispatch!(self, executed_epoch)
    }

    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)> {
        dispatch!(self, modified_at_versions)
    }

    fn lamport_version(&self) -> Version {
        dispatch!(self, lamport_version)
    }

    fn old_object_metadata(&self) -> Vec<(ObjectRef, Owner)> {
        dispatch!(self, old_object_metadata)
    }

    fn sequenced_input_shared_objects(&self) -> Vec<InputSharedObject> {
        dispatch!(self, sequenced_input_shared_objects)
    }

    fn created(&self) -> Vec<(ObjectRef, Owner)> {
        dispatch!(self, created)
    }

    fn mutated(&self) -> Vec<(ObjectRef, Owner)> {
        dispatch!(self, mutated)
    }

    fn unwrapped(&self) -> Vec<(ObjectRef, Owner)> {
        dispatch!(self, unwrapped)
    }

    fn deleted(&self) -> Vec<ObjectRef> {
        dispatch!(self, deleted)
    }

    fn unwrapped_then_deleted(&self) -> Vec<ObjectRef> {
        dispatch!(self, unwrapped_then_deleted)
    }

    fn wrapped(&self) -> Vec<ObjectRef> {
        dispatch!(self, wrapped)
    }

    fn object_changes(&self) -> Vec<ObjectChange> {
        dispatch!(self, object_changes)
    }

    fn gas_object(&self) -> Option<(ObjectRef, Owner)> {
        dispatch!(self, gas_object)
    }

    fn events_digest(&self) -> Option<&TransactionEventsDigest> {
        dispatch!(self, events_digest)
    }

    fn dependencies(&self) -> &[TransactionDigest] {
        dispatch!(self, dependencies)
    }

    fn transaction_digest(&self) -> &TransactionDigest {
        dispatch!(self, transaction_digest)
    }

    fn gas_cost_summary(&self) -> &GasCostSummary {
        dispatch!(self, gas_cost_summary)
    }

    fn unchanged_shared_objects(&self) -> Vec<(ObjectId, UnchangedSharedKind)> {
        dispatch!(self, unchanged_shared_objects)
    }
}

impl TransactionEffectsAPI for TransactionEffectsV1 {
    fn status(&self) -> &ExecutionStatus {
        &self.status
    }

    fn into_status(self) -> ExecutionStatus {
        self.status
    }

    fn executed_epoch(&self) -> EpochId {
        self.epoch
    }

    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)> {
        self.modified_at_versions
            .iter()
            // V1 transaction effects "modified_at_versions" includes unwrapped_then_deleted
            // objects, so in order to have parity with the V2 transaction effects semantics of
            // "modified_at_versions", filter out any objects that are unwrapped_then_deleted'ed
            .filter(|key| {
                !self
                    .unwrapped_then_deleted
                    .iter()
                    .any(|deleted| deleted.object_id() == &key.object_id)
            })
            .map(|key| (key.object_id, key.version))
            .collect()
    }

    fn lamport_version(&self) -> Version {
        self.modified_at_versions
            .iter()
            .map(|key| key.version)
            .fold(0, std::cmp::max)
            + 1
    }

    fn old_object_metadata(&self) -> Vec<(ObjectRef, Owner)> {
        unimplemented!("Only supposed by v2 and above");
    }

    fn sequenced_input_shared_objects(&self) -> Vec<InputSharedObject> {
        let modified: HashSet<_> = self
            .modified_at_versions
            .iter()
            .map(|key| key.object_id)
            .collect();
        self.shared_objects
            .iter()
            .cloned()
            .map(ObjectReference::into_parts)
            .map(|r| {
                if modified.contains(&r.0) {
                    InputSharedObject::Mutate(r)
                } else {
                    InputSharedObject::ReadOnly(r)
                }
            })
            .collect()
    }

    fn created(&self) -> Vec<(ObjectRef, Owner)> {
        self.created.iter().cloned().map(into_parts).collect()
    }

    fn mutated(&self) -> Vec<(ObjectRef, Owner)> {
        self.mutated.iter().cloned().map(into_parts).collect()
    }

    fn unwrapped(&self) -> Vec<(ObjectRef, Owner)> {
        self.unwrapped.iter().cloned().map(into_parts).collect()
    }

    fn deleted(&self) -> Vec<ObjectRef> {
        self.deleted
            .iter()
            .cloned()
            .map(ObjectReference::into_parts)
            .collect()
    }

    fn unwrapped_then_deleted(&self) -> Vec<ObjectRef> {
        self.unwrapped_then_deleted
            .iter()
            .cloned()
            .map(ObjectReference::into_parts)
            .collect()
    }

    fn wrapped(&self) -> Vec<ObjectRef> {
        self.wrapped
            .iter()
            .cloned()
            .map(ObjectReference::into_parts)
            .collect()
    }

    fn object_changes(&self) -> Vec<ObjectChange> {
        let modified_at: BTreeMap<_, _> = self
            .modified_at_versions
            .iter()
            .map(|m| (m.object_id, m.version))
            .collect();

        let created = self.created.iter().map(|r| ObjectChange {
            id: *r.reference.object_id(),
            input_version: None,
            input_digest: None,
            output_version: Some(r.reference.version()),
            output_digest: Some(*r.reference.digest()),
            id_operation: IdOperation::Created,
        });

        let mutated = self.mutated.iter().map(|r| ObjectChange {
            id: *r.reference.object_id(),
            input_version: modified_at.get(r.reference.object_id()).copied(),
            input_digest: None,
            output_version: Some(r.reference.version()),
            output_digest: Some(*r.reference.digest()),
            id_operation: IdOperation::None,
        });

        let unwrapped = self.unwrapped.iter().map(|r| ObjectChange {
            id: *r.reference.object_id(),
            input_version: None,
            input_digest: None,
            output_version: Some(r.reference.version()),
            output_digest: Some(*r.reference.digest()),
            id_operation: IdOperation::None,
        });

        let deleted = self.deleted.iter().map(|r| ObjectChange {
            id: *r.object_id(),
            input_version: modified_at.get(r.object_id()).copied(),
            input_digest: None,
            output_version: None,
            output_digest: None,
            id_operation: IdOperation::Deleted,
        });

        let unwrapped_then_deleted = self.unwrapped_then_deleted.iter().map(|r| ObjectChange {
            id: *r.object_id(),
            input_version: None,
            input_digest: None,
            output_version: None,
            output_digest: None,
            id_operation: IdOperation::Deleted,
        });

        let wrapped = self.wrapped.iter().map(|r| ObjectChange {
            id: *r.object_id(),
            input_version: modified_at.get(r.object_id()).copied(),
            input_digest: None,
            output_version: None,
            output_digest: None,
            id_operation: IdOperation::None,
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
        Some(into_parts(self.gas_object.clone()))
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
                InputSharedObject::ReadOnly(oref) => Some((
                    oref.0,
                    UnchangedSharedKind::ReadOnlyRoot {
                        version: oref.1,
                        digest: oref.2,
                    },
                )),
                _ => None,
            })
            .collect()
    }
}

impl TransactionEffectsAPI for TransactionEffectsV2 {
    fn status(&self) -> &ExecutionStatus {
        &self.status
    }

    fn into_status(self) -> ExecutionStatus {
        self.status
    }

    fn executed_epoch(&self) -> EpochId {
        self.epoch
    }

    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)> {
        self.changed_objects
            .iter()
            .filter_map(|c| {
                if let ObjectIn::Exist { version, .. } = &c.input_state {
                    Some((c.object_id, *version))
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
            .filter_map(|c| {
                if let ObjectIn::Exist {
                    version,
                    digest,
                    owner,
                } = c.input_state
                {
                    Some(((c.object_id, version, digest), owner.into()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn sequenced_input_shared_objects(&self) -> Vec<InputSharedObject> {
        self.changed_objects
            .iter()
            .filter_map(|c| match c.input_state {
                ObjectIn::Exist {
                    version, digest, ..
                } => Some(InputSharedObject::Mutate((c.object_id, version, digest))),
                _ => None,
            })
            .chain(
                self.unchanged_shared_objects
                    .iter()
                    .filter_map(|u| match u.kind {
                        UnchangedSharedKind::ReadOnlyRoot { version, digest } => {
                            Some(InputSharedObject::ReadOnly((u.object_id, version, digest)))
                        }
                        UnchangedSharedKind::MutateDeleted { version } => {
                            Some(InputSharedObject::MutateDeleted(u.object_id, version))
                        }
                        UnchangedSharedKind::ReadDeleted { version } => {
                            Some(InputSharedObject::ReadDeleted(u.object_id, version))
                        }
                        UnchangedSharedKind::Cancelled { version } => {
                            Some(InputSharedObject::Cancelled(u.object_id, version))
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
            .filter_map(
                |c| match (&c.input_state, &c.output_state, c.id_operation) {
                    (
                        ObjectIn::NotExist,
                        ObjectOut::ObjectWrite { digest, owner },
                        IdOperation::Created,
                    ) => Some((
                        (c.object_id, self.lamport_version, *digest),
                        (*owner).into(),
                    )),
                    (
                        ObjectIn::NotExist,
                        ObjectOut::PackageWrite { version, digest },
                        IdOperation::Created,
                    ) => Some(((c.object_id, *version, *digest), Owner::Immutable)),
                    _ => None,
                },
            )
            .collect()
    }

    fn mutated(&self) -> Vec<(ObjectRef, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(|c| match (&c.input_state, &c.output_state) {
                (ObjectIn::Exist { .. }, ObjectOut::ObjectWrite { digest, owner }) => Some((
                    (c.object_id, self.lamport_version, *digest),
                    (*owner).into(),
                )),
                (ObjectIn::Exist { .. }, ObjectOut::PackageWrite { version, digest }) => {
                    Some(((c.object_id, *version, *digest), Owner::Immutable))
                }
                _ => None,
            })
            .collect()
    }

    fn unwrapped(&self) -> Vec<(ObjectRef, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(
                |c| match (&c.input_state, &c.output_state, &c.id_operation) {
                    (
                        ObjectIn::NotExist,
                        ObjectOut::ObjectWrite { digest, owner },
                        IdOperation::None,
                    ) => Some((
                        (c.object_id, self.lamport_version, *digest),
                        (*owner).into(),
                    )),
                    _ => None,
                },
            )
            .collect()
    }

    fn deleted(&self) -> Vec<ObjectRef> {
        self.changed_objects
            .iter()
            .filter_map(
                |c| match (&c.input_state, &c.output_state, &c.id_operation) {
                    (ObjectIn::Exist { .. }, ObjectOut::NotExist, IdOperation::Deleted) => {
                        Some((c.object_id, self.lamport_version, OBJECT_DIGEST_DELETED))
                    }
                    _ => None,
                },
            )
            .collect()
    }

    fn unwrapped_then_deleted(&self) -> Vec<ObjectRef> {
        self.changed_objects
            .iter()
            .filter_map(
                |c| match (&c.input_state, &c.output_state, &c.id_operation) {
                    (ObjectIn::NotExist, ObjectOut::NotExist, IdOperation::Deleted) => {
                        Some((c.object_id, self.lamport_version, OBJECT_DIGEST_DELETED))
                    }
                    _ => None,
                },
            )
            .collect()
    }

    fn wrapped(&self) -> Vec<ObjectRef> {
        self.changed_objects
            .iter()
            .filter_map(
                |c| match (&c.input_state, &c.output_state, &c.id_operation) {
                    (ObjectIn::Exist { .. }, ObjectOut::NotExist, IdOperation::None) => {
                        Some((c.object_id, self.lamport_version, OBJECT_DIGEST_WRAPPED))
                    }
                    _ => None,
                },
            )
            .collect()
    }

    fn object_changes(&self) -> Vec<ObjectChange> {
        self.changed_objects
            .iter()
            .map(|c| {
                let input_version_digest = match &c.input_state {
                    ObjectIn::NotExist => None,
                    ObjectIn::Exist {
                        version, digest, ..
                    } => Some((*version, *digest)),
                };

                let output_version_digest = match &c.output_state {
                    ObjectOut::NotExist => None,
                    ObjectOut::ObjectWrite { digest, .. } => Some((self.lamport_version, *digest)),
                    ObjectOut::PackageWrite {
                        version, digest, ..
                    } => Some((*version, *digest)),
                };

                ObjectChange {
                    id: c.object_id,

                    input_version: input_version_digest.map(|k| k.0),
                    input_digest: input_version_digest.map(|k| k.1),

                    output_version: output_version_digest.map(|k| k.0),
                    output_digest: output_version_digest.map(|k| k.1),

                    id_operation: c.id_operation,
                }
            })
            .collect()
    }

    fn gas_object(&self) -> Option<(ObjectRef, Owner)> {
        self.gas_object_index.map(|gas_object_index| {
            let entry = &self.changed_objects[gas_object_index as usize];
            match entry.output_state {
                ObjectOut::ObjectWrite { digest, owner } => (
                    (entry.object_id, self.lamport_version, digest),
                    owner.into(),
                ),
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
        self.unchanged_shared_objects
            .clone()
            .into_iter()
            .map(|u| (u.object_id, u.kind))
            .collect()
    }
}

fn into_parts(r: ObjectReferenceWithOwner) -> (ObjectRef, Owner) {
    (r.reference.into_parts(), r.owner.into())
}
