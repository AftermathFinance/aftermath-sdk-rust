// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{self, Display, Formatter, Write};
use std::str::FromStr;

use af_sui_types::{
    Address as SuiAddress,
    CheckpointDigest,
    ConsensusCommitDigest,
    EpochId,
    GasCostSummary,
    ObjectDigest,
    ObjectId,
    ObjectRef,
    Owner,
    StructTag,
    TransactionDigest,
    TransactionEventsDigest,
    TypeTag,
    SUI_FRAMEWORK_ADDRESS,
};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::{serde_as, DisplayFromStr, IfIsHumanReadable};
use sui_sdk_types::{ConsensusDeterminedVersionAssignments, MoveLocation, UserSignature, Version};
use tabled::builder::Builder as TableBuilder;
use tabled::settings::style::HorizontalLine;
use tabled::settings::{Panel as TablePanel, Style as TableStyle};

use super::balance_changes::BalanceChange;
use super::object_changes::ObjectChange;
use super::{Page, SuiEvent, SuiObjectRef};
use crate::serde::BigInt;

/// similar to EpochId of sui-types but BigInt
pub type SuiEpochId = BigInt<u64>;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(
    rename_all = "camelCase",
    rename = "TransactionBlockResponseQuery",
    default
)]
pub struct SuiTransactionBlockResponseQuery {
    /// If None, no filter will be applied
    pub filter: Option<TransactionFilter>,
    /// config which fields to include in the response, by default only digest is included
    pub options: Option<SuiTransactionBlockResponseOptions>,
}

impl SuiTransactionBlockResponseQuery {
    pub fn new(
        filter: Option<TransactionFilter>,
        options: Option<SuiTransactionBlockResponseOptions>,
    ) -> Self {
        Self { filter, options }
    }

    pub fn new_with_filter(filter: TransactionFilter) -> Self {
        Self {
            filter: Some(filter),
            options: None,
        }
    }
}

pub type TransactionBlocksPage = Page<SuiTransactionBlockResponse, TransactionDigest>;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, Default)]
#[serde(
    rename_all = "camelCase",
    rename = "TransactionBlockResponseOptions",
    default
)]
pub struct SuiTransactionBlockResponseOptions {
    /// Whether to show transaction input data. Default to be False
    pub show_input: bool,
    /// Whether to show bcs-encoded transaction input data
    pub show_raw_input: bool,
    /// Whether to show transaction effects. Default to be False
    pub show_effects: bool,
    /// Whether to show transaction events. Default to be False
    pub show_events: bool,
    /// Whether to show object_changes. Default to be False
    pub show_object_changes: bool,
    /// Whether to show balance_changes. Default to be False
    pub show_balance_changes: bool,
    /// Whether to show raw transaction effects. Default to be False
    pub show_raw_effects: bool,
}

impl SuiTransactionBlockResponseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn full_content() -> Self {
        Self {
            show_effects: true,
            show_input: true,
            show_raw_input: true,
            show_events: true,
            show_object_changes: true,
            show_balance_changes: true,
            // This field is added for graphql execution. We keep it false here
            // so current users of `full_content` will not get raw effects unexpectedly.
            show_raw_effects: false,
        }
    }

    pub fn with_input(mut self) -> Self {
        self.show_input = true;
        self
    }

    pub fn with_raw_input(mut self) -> Self {
        self.show_raw_input = true;
        self
    }

    pub fn with_effects(mut self) -> Self {
        self.show_effects = true;
        self
    }

    pub fn with_events(mut self) -> Self {
        self.show_events = true;
        self
    }

    pub fn with_balance_changes(mut self) -> Self {
        self.show_balance_changes = true;
        self
    }

    pub fn with_object_changes(mut self) -> Self {
        self.show_object_changes = true;
        self
    }

    pub fn with_raw_effects(mut self) -> Self {
        self.show_raw_effects = true;
        self
    }

    /// default to return `WaitForEffectsCert` unless some options require
    /// local execution
    pub fn default_execution_request_type(&self) -> ExecuteTransactionRequestType {
        // if people want effects or events, they typically want to wait for local execution
        if self.require_effects() {
            ExecuteTransactionRequestType::WaitForLocalExecution
        } else {
            ExecuteTransactionRequestType::WaitForEffectsCert
        }
    }

    pub fn require_input(&self) -> bool {
        self.show_input || self.show_raw_input || self.show_object_changes
    }

    pub fn require_effects(&self) -> bool {
        self.show_effects
            || self.show_events
            || self.show_balance_changes
            || self.show_object_changes
            || self.show_raw_effects
    }

    pub fn only_digest(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExecuteTransactionRequestType {
    /// Waits for `TransactionEffectsCert` and then return to client. This mode is a proxy for
    /// transaction finality.
    WaitForEffectsCert,
    /// JSON-RPC now ignores this. It will always behave as if
    /// [`WaitForEffectsCert`](ExecuteTransactionRequestType::WaitForEffectsCert) was passed.
    ///
    /// Originally: waits for `TransactionEffectsCert` and make sure the node executed the
    /// transaction locally before returning the client. The local execution makes sure this node is
    /// aware of this transaction when client fires subsequent queries. However if the node fails to
    /// execute the transaction locally in a timely manner, a bool type in the response is set to
    /// false to indicated the case. request_type is default to be `WaitForEffectsCert` unless
    /// options.show_events or options.show_effects is true
    WaitForLocalExecution,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", rename = "TransactionBlockResponse")]
pub struct SuiTransactionBlockResponse {
    pub digest: TransactionDigest,
    /// Transaction input data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<SuiTransactionBlock>,
    /// BCS encoded [SenderSignedData](https://mystenlabs.github.io/sui/sui_types/transaction/struct.SenderSignedData.html)
    /// that includes input object references returns empty array if `show_raw_transaction` is false
    #[serde_as(as = "Base64")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_transaction: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<SuiTransactionBlockEffects>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<SuiTransactionBlockEvents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_changes: Option<Vec<ObjectChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_changes: Option<Vec<BalanceChange>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<BigInt<u64>>")]
    pub timestamp_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_local_execution: Option<bool>,
    /// The checkpoint number when this transaction was included and hence finalized.
    /// This is only returned in the read api, not in the transaction execution api.
    #[serde_as(as = "Option<BigInt<u64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<Version>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_effects: Vec<u8>,
}

impl SuiTransactionBlockResponse {
    pub fn new(digest: TransactionDigest) -> Self {
        Self {
            digest,
            ..Default::default()
        }
    }

    pub fn status_ok(&self) -> Option<bool> {
        self.effects.as_ref().map(|e| e.status().is_ok())
    }

    pub fn get_transaction(
        &self,
    ) -> Result<&SuiTransactionBlock, SuiTransactionBlockResponseError> {
        self.transaction
            .as_ref()
            .ok_or(SuiTransactionBlockResponseError::MissingTransaction)
    }

    pub fn get_effects(
        &self,
    ) -> Result<&SuiTransactionBlockEffectsV1, SuiTransactionBlockResponseError> {
        let SuiTransactionBlockEffects::V1(ref effects) = self
            .effects
            .as_ref()
            .ok_or(SuiTransactionBlockResponseError::MissingEffects)?;
        Ok(effects)
    }

    pub fn get_events(
        &self,
    ) -> Result<&SuiTransactionBlockEvents, SuiTransactionBlockResponseError> {
        self.events
            .as_ref()
            .ok_or(SuiTransactionBlockResponseError::MissingEvents)
    }

    pub fn get_object_changes(
        &self,
    ) -> Result<&Vec<ObjectChange>, SuiTransactionBlockResponseError> {
        self.object_changes
            .as_ref()
            .ok_or(SuiTransactionBlockResponseError::MissingObjectChanges)
    }

    pub fn get_balance_changes(
        &self,
    ) -> Result<&Vec<BalanceChange>, SuiTransactionBlockResponseError> {
        self.balance_changes
            .as_ref()
            .ok_or(SuiTransactionBlockResponseError::MissingBalanceChanges)
    }

    pub fn try_check_execution_status(&self) -> Result<(), SuiTransactionBlockResponseError> {
        if let Some(SuiTransactionBlockEffects::V1(effects)) = &self.effects {
            if let SuiExecutionStatus::Failure { error } = &effects.status {
                return Err(SuiTransactionBlockResponseError::ExecutionFailure(
                    error.clone(),
                ));
            }
        }
        Ok(())
    }

    pub fn check_execution_status(&self) -> Result<(), SuiTransactionBlockResponseError> {
        let Some(SuiTransactionBlockEffects::V1(effects)) = &self.effects else {
            return Err(SuiTransactionBlockResponseError::MissingEffects);
        };
        if let SuiExecutionStatus::Failure { error } = &effects.status {
            return Err(SuiTransactionBlockResponseError::ExecutionFailure(
                error.clone(),
            ));
        }
        Ok(())
    }

    pub fn published_package_id(&self) -> Result<ObjectId, SuiTransactionBlockResponseError> {
        for change in self.get_object_changes()? {
            if let ObjectChange::Published { package_id, .. } = change {
                return Ok(*package_id);
            }
        }
        Err(SuiTransactionBlockResponseError::NoPublishedPackage)
    }

    pub fn into_object_changes(
        self,
    ) -> Result<Vec<ObjectChange>, SuiTransactionBlockResponseError> {
        let Self { object_changes, .. } = self;
        object_changes.ok_or(SuiTransactionBlockResponseError::MissingObjectChanges)
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum SuiTransactionBlockResponseError {
    #[error("No transaction in response")]
    MissingTransaction,
    #[error("No effects in response")]
    MissingEffects,
    #[error("No events in response")]
    MissingEvents,
    #[error("No object changes in response")]
    MissingObjectChanges,
    #[error("No balance changes in response")]
    MissingBalanceChanges,
    #[error("Failed to execute transaction block: {0}")]
    ExecutionFailure(String),
    #[error("No 'Published' object change")]
    NoPublishedPackage,
}

/// We are specifically ignoring events for now until events become more stable.
impl PartialEq for SuiTransactionBlockResponse {
    fn eq(&self, other: &Self) -> bool {
        self.transaction == other.transaction
            && self.effects == other.effects
            && self.timestamp_ms == other.timestamp_ms
            && self.confirmed_local_execution == other.confirmed_local_execution
            && self.checkpoint == other.checkpoint
    }
}

impl Display for SuiTransactionBlockResponse {
    fn fmt(&self, writer: &mut Formatter<'_>) -> fmt::Result {
        writeln!(writer, "Transaction Digest: {}", &self.digest)?;

        if let Some(t) = &self.transaction {
            writeln!(writer, "{}", t)?;
        }

        if let Some(e) = &self.effects {
            writeln!(writer, "{}", e)?;
        }

        if let Some(e) = &self.events {
            writeln!(writer, "{}", e)?;
        }

        if let Some(object_changes) = &self.object_changes {
            let mut builder = TableBuilder::default();
            let (
                mut created,
                mut deleted,
                mut mutated,
                mut published,
                mut transferred,
                mut wrapped,
            ) = (vec![], vec![], vec![], vec![], vec![], vec![]);

            for obj in object_changes {
                match obj {
                    ObjectChange::Created { .. } => created.push(obj),
                    ObjectChange::Deleted { .. } => deleted.push(obj),
                    ObjectChange::Mutated { .. } => mutated.push(obj),
                    ObjectChange::Published { .. } => published.push(obj),
                    ObjectChange::Transferred { .. } => transferred.push(obj),
                    ObjectChange::Wrapped { .. } => wrapped.push(obj),
                };
            }

            write_obj_changes(created, "Created", &mut builder)?;
            write_obj_changes(deleted, "Deleted", &mut builder)?;
            write_obj_changes(mutated, "Mutated", &mut builder)?;
            write_obj_changes(published, "Published", &mut builder)?;
            write_obj_changes(transferred, "Transferred", &mut builder)?;
            write_obj_changes(wrapped, "Wrapped", &mut builder)?;

            let mut table = builder.build();
            table.with(TablePanel::header("Object Changes"));
            table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
                1,
                TableStyle::modern().get_horizontal(),
            )]));
            writeln!(writer, "{}", table)?;
        }

        if let Some(balance_changes) = &self.balance_changes {
            let mut builder = TableBuilder::default();
            for balance in balance_changes {
                builder.push_record(vec![format!("{}", balance)]);
            }
            let mut table = builder.build();
            table.with(TablePanel::header("Balance Changes"));
            table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
                1,
                TableStyle::modern().get_horizontal(),
            )]));
            writeln!(writer, "{}", table)?;
        }
        Ok(())
    }
}

fn write_obj_changes<T: Display>(
    values: Vec<T>,
    output_string: &str,
    builder: &mut TableBuilder,
) -> std::fmt::Result {
    if !values.is_empty() {
        builder.push_record(vec![format!("{} Objects: ", output_string)]);
        for obj in values {
            builder.push_record(vec![format!("{}", obj)]);
        }
    }
    Ok(())
}

pub fn get_new_package_obj_from_response(
    response: &SuiTransactionBlockResponse,
) -> Option<ObjectRef> {
    response.object_changes.as_ref().and_then(|changes| {
        changes
            .iter()
            .find(|change| matches!(change, ObjectChange::Published { .. }))
            .map(|change| change.object_ref())
    })
}

pub fn get_new_package_upgrade_cap_from_response(
    response: &SuiTransactionBlockResponse,
) -> Option<ObjectRef> {
    response.object_changes.as_ref().and_then(|changes| {
        changes
            .iter()
            .find(|change| {
                matches!(change, ObjectChange::Created {
                    owner: Owner::AddressOwner(_),
                    object_type: StructTag {
                        address: SUI_FRAMEWORK_ADDRESS,
                        module,
                        name,
                        ..
                    },
                    ..
                } if module.as_str() == "package" && name.as_str() == "UpgradeCap")
            })
            .map(|change| change.object_ref())
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "TransactionBlockKind", tag = "kind")]
#[non_exhaustive]
pub enum SuiTransactionBlockKind {
    /// A system transaction that will update epoch information on-chain.
    ChangeEpoch(SuiChangeEpoch),
    /// A system transaction used for initializing the initial state of the chain.
    Genesis(SuiGenesisTransaction),
    /// A system transaction marking the start of a series of transactions scheduled as part of a
    /// checkpoint
    ConsensusCommitPrologue(SuiConsensusCommitPrologue),
    /// A series of transactions where the results of one transaction can be used in future
    /// transactions
    ProgrammableTransaction(SuiProgrammableTransactionBlock),
    /// A transaction which updates global authenticator state
    AuthenticatorStateUpdate(SuiAuthenticatorStateUpdate),
    /// A transaction which updates global randomness state
    RandomnessStateUpdate(SuiRandomnessStateUpdate),
    /// The transaction which occurs only at the end of the epoch
    EndOfEpochTransaction(SuiEndOfEpochTransaction),
    ConsensusCommitPrologueV2(SuiConsensusCommitPrologueV2),
    ConsensusCommitPrologueV3(SuiConsensusCommitPrologueV3),
    // .. more transaction types go here
}

impl Display for SuiTransactionBlockKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        match &self {
            Self::ChangeEpoch(e) => {
                writeln!(writer, "Transaction Kind: Epoch Change")?;
                writeln!(writer, "New epoch ID: {}", e.epoch)?;
                writeln!(writer, "Storage gas reward: {}", e.storage_charge)?;
                writeln!(writer, "Computation gas reward: {}", e.computation_charge)?;
                writeln!(writer, "Storage rebate: {}", e.storage_rebate)?;
                writeln!(writer, "Timestamp: {}", e.epoch_start_timestamp_ms)?;
            }
            Self::Genesis(_) => {
                writeln!(writer, "Transaction Kind: Genesis Transaction")?;
            }
            Self::ConsensusCommitPrologue(p) => {
                writeln!(writer, "Transaction Kind: Consensus Commit Prologue")?;
                writeln!(
                    writer,
                    "Epoch: {}, Round: {}, Timestamp: {}",
                    p.epoch, p.round, p.commit_timestamp_ms
                )?;
            }
            Self::ConsensusCommitPrologueV2(p) => {
                writeln!(writer, "Transaction Kind: Consensus Commit Prologue V2")?;
                writeln!(
                    writer,
                    "Epoch: {}, Round: {}, Timestamp: {}, ConsensusCommitDigest: {}",
                    p.epoch, p.round, p.commit_timestamp_ms, p.consensus_commit_digest
                )?;
            }
            Self::ConsensusCommitPrologueV3(p) => {
                writeln!(writer, "Transaction Kind: Consensus Commit Prologue V3")?;
                writeln!(
                    writer,
                    "Epoch: {}, Round: {}, SubDagIndex: {:?}, Timestamp: {}, ConsensusCommitDigest: {}",
                    p.epoch, p.round, p.sub_dag_index, p.commit_timestamp_ms, p.consensus_commit_digest
                )?;
            }
            Self::ProgrammableTransaction(p) => {
                write!(writer, "Transaction Kind: Programmable")?;
                write!(writer, "{}", super::displays::Pretty(p))?;
            }
            Self::AuthenticatorStateUpdate(_) => {
                writeln!(writer, "Transaction Kind: Authenticator State Update")?;
            }
            Self::RandomnessStateUpdate(_) => {
                writeln!(writer, "Transaction Kind: Randomness State Update")?;
            }
            Self::EndOfEpochTransaction(_) => {
                writeln!(writer, "Transaction Kind: End of Epoch Transaction")?;
            }
        }
        write!(f, "{}", writer)
    }
}

impl SuiTransactionBlockKind {
    pub fn transaction_count(&self) -> usize {
        match self {
            Self::ProgrammableTransaction(p) => p.commands.len(),
            _ => 1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ChangeEpoch(_) => "ChangeEpoch",
            Self::Genesis(_) => "Genesis",
            Self::ConsensusCommitPrologue(_) => "ConsensusCommitPrologue",
            Self::ConsensusCommitPrologueV2(_) => "ConsensusCommitPrologueV2",
            Self::ConsensusCommitPrologueV3(_) => "ConsensusCommitPrologueV3",
            Self::ProgrammableTransaction(_) => "ProgrammableTransaction",
            Self::AuthenticatorStateUpdate(_) => "AuthenticatorStateUpdate",
            Self::RandomnessStateUpdate(_) => "RandomnessStateUpdate",
            Self::EndOfEpochTransaction(_) => "EndOfEpochTransaction",
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiChangeEpoch {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: EpochId,
    #[serde_as(as = "BigInt<u64>")]
    pub storage_charge: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub computation_charge: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub storage_rebate: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub epoch_start_timestamp_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[enum_dispatch(SuiTransactionBlockEffectsAPI)]
#[serde(
    rename = "TransactionBlockEffects",
    rename_all = "camelCase",
    tag = "messageVersion"
)]
pub enum SuiTransactionBlockEffects {
    V1(SuiTransactionBlockEffectsV1),
}

#[enum_dispatch]
pub trait SuiTransactionBlockEffectsAPI {
    fn status(&self) -> &SuiExecutionStatus;
    fn into_status(self) -> SuiExecutionStatus;
    fn shared_objects(&self) -> &[SuiObjectRef];
    fn created(&self) -> &[OwnedObjectRef];
    fn mutated(&self) -> &[OwnedObjectRef];
    fn unwrapped(&self) -> &[OwnedObjectRef];
    fn deleted(&self) -> &[SuiObjectRef];
    fn unwrapped_then_deleted(&self) -> &[SuiObjectRef];
    fn wrapped(&self) -> &[SuiObjectRef];
    fn gas_object(&self) -> &OwnedObjectRef;
    fn events_digest(&self) -> Option<&TransactionEventsDigest>;
    fn dependencies(&self) -> &[TransactionDigest];
    fn executed_epoch(&self) -> EpochId;
    fn transaction_digest(&self) -> &TransactionDigest;
    fn gas_cost_summary(&self) -> &GasCostSummary;

    /// Return an iterator of mutated objects, but excluding the gas object.
    fn mutated_excluding_gas(&self) -> Vec<OwnedObjectRef>;
    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)>;
    fn all_changed_objects(&self) -> Vec<(&OwnedObjectRef, WriteKind)>;
    fn all_deleted_objects(&self) -> Vec<(&SuiObjectRef, DeleteKind)>;
}

/// Originally from `sui_types::storage`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum WriteKind {
    /// The object was in storage already but has been modified
    Mutate,
    /// The object was created in this transaction
    Create,
    /// The object was previously wrapped in another object, but has been restored to storage
    Unwrap,
}

/// Originally from `sui_types::storage`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum DeleteKind {
    /// An object is provided in the call input, and gets deleted.
    Normal,
    /// An object is not provided in the call input, but gets unwrapped
    /// from another object, and then gets deleted.
    UnwrapThenDelete,
    /// An object is provided in the call input, and gets wrapped into another object.
    Wrap,
}

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(
    rename = "TransactionBlockEffectsModifiedAtVersions",
    rename_all = "camelCase"
)]
pub struct SuiTransactionBlockEffectsModifiedAtVersions {
    object_id: ObjectId,
    #[serde_as(as = "BigInt<u64>")]
    sequence_number: Version,
}

/// The response from processing a transaction or a certified transaction
#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "TransactionBlockEffectsV1", rename_all = "camelCase")]
pub struct SuiTransactionBlockEffectsV1 {
    /// The status of the execution
    pub status: SuiExecutionStatus,
    /// The epoch when this transaction was executed.
    #[serde_as(as = "BigInt<u64>")]
    pub executed_epoch: EpochId,
    #[serde_as(as = "serde_with::FromInto<crate::serde::GasCostSummaryJson>")]
    pub gas_used: GasCostSummary,
    /// The version that every modified (mutated or deleted) object had before it was modified by
    /// this transaction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modified_at_versions: Vec<SuiTransactionBlockEffectsModifiedAtVersions>,
    /// The object references of the shared objects used in this transaction. Empty if no shared objects were used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_objects: Vec<SuiObjectRef>,
    /// The transaction digest
    pub transaction_digest: TransactionDigest,
    /// ObjectRef and owner of new objects created.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub created: Vec<OwnedObjectRef>,
    /// ObjectRef and owner of mutated objects, including gas object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutated: Vec<OwnedObjectRef>,
    /// ObjectRef and owner of objects that are unwrapped in this transaction.
    /// Unwrapped objects are objects that were wrapped into other objects in the past,
    /// and just got extracted out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unwrapped: Vec<OwnedObjectRef>,
    /// Object Refs of objects now deleted (the old refs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deleted: Vec<SuiObjectRef>,
    /// Object refs of objects previously wrapped in other objects but now deleted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unwrapped_then_deleted: Vec<SuiObjectRef>,
    /// Object refs of objects now wrapped in other objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wrapped: Vec<SuiObjectRef>,
    /// The updated gas object reference. Have a dedicated field for convenient access.
    /// It's also included in mutated.
    pub gas_object: OwnedObjectRef,
    /// The digest of the events emitted during execution,
    /// can be None if the transaction does not emit any event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_digest: Option<TransactionEventsDigest>,
    /// The set of transaction digests this transaction depends on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<TransactionDigest>,
}

impl SuiTransactionBlockEffectsAPI for SuiTransactionBlockEffectsV1 {
    fn status(&self) -> &SuiExecutionStatus {
        &self.status
    }
    fn into_status(self) -> SuiExecutionStatus {
        self.status
    }
    fn shared_objects(&self) -> &[SuiObjectRef] {
        &self.shared_objects
    }
    fn created(&self) -> &[OwnedObjectRef] {
        &self.created
    }
    fn mutated(&self) -> &[OwnedObjectRef] {
        &self.mutated
    }
    fn unwrapped(&self) -> &[OwnedObjectRef] {
        &self.unwrapped
    }
    fn deleted(&self) -> &[SuiObjectRef] {
        &self.deleted
    }
    fn unwrapped_then_deleted(&self) -> &[SuiObjectRef] {
        &self.unwrapped_then_deleted
    }
    fn wrapped(&self) -> &[SuiObjectRef] {
        &self.wrapped
    }
    fn gas_object(&self) -> &OwnedObjectRef {
        &self.gas_object
    }
    fn events_digest(&self) -> Option<&TransactionEventsDigest> {
        self.events_digest.as_ref()
    }
    fn dependencies(&self) -> &[TransactionDigest] {
        &self.dependencies
    }

    fn executed_epoch(&self) -> EpochId {
        self.executed_epoch
    }

    fn transaction_digest(&self) -> &TransactionDigest {
        &self.transaction_digest
    }

    fn gas_cost_summary(&self) -> &GasCostSummary {
        &self.gas_used
    }

    fn mutated_excluding_gas(&self) -> Vec<OwnedObjectRef> {
        self.mutated
            .iter()
            .filter(|o| *o != &self.gas_object)
            .cloned()
            .collect()
    }

    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)> {
        self.modified_at_versions
            .iter()
            .map(|v| (v.object_id, v.sequence_number))
            .collect::<Vec<_>>()
    }

    fn all_changed_objects(&self) -> Vec<(&OwnedObjectRef, WriteKind)> {
        self.mutated
            .iter()
            .map(|owner_ref| (owner_ref, WriteKind::Mutate))
            .chain(
                self.created
                    .iter()
                    .map(|owner_ref| (owner_ref, WriteKind::Create)),
            )
            .chain(
                self.unwrapped
                    .iter()
                    .map(|owner_ref| (owner_ref, WriteKind::Unwrap)),
            )
            .collect()
    }

    fn all_deleted_objects(&self) -> Vec<(&SuiObjectRef, DeleteKind)> {
        self.deleted
            .iter()
            .map(|r| (r, DeleteKind::Normal))
            .chain(
                self.unwrapped_then_deleted
                    .iter()
                    .map(|r| (r, DeleteKind::UnwrapThenDelete)),
            )
            .chain(self.wrapped.iter().map(|r| (r, DeleteKind::Wrap)))
            .collect()
    }
}

fn owned_objref_string(obj: &OwnedObjectRef) -> String {
    format!(
        " ┌──\n │ ID: {} \n │ Owner: {} \n │ Version: {} \n │ Digest: {}\n └──",
        obj.reference.object_id, obj.owner, obj.reference.version, obj.reference.digest
    )
}

fn objref_string(obj: &SuiObjectRef) -> String {
    format!(
        " ┌──\n │ ID: {} \n │ Version: {} \n │ Digest: {}\n └──",
        obj.object_id, obj.version, obj.digest
    )
}

impl Display for SuiTransactionBlockEffects {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut builder = TableBuilder::default();

        builder.push_record(vec![format!("Digest: {}", self.transaction_digest())]);
        builder.push_record(vec![format!("Status: {:?}", self.status())]);
        builder.push_record(vec![format!("Executed Epoch: {}", self.executed_epoch())]);

        if !self.created().is_empty() {
            builder.push_record(vec![format!("\nCreated Objects: ")]);

            for oref in self.created() {
                builder.push_record(vec![owned_objref_string(oref)]);
            }
        }

        if !self.mutated().is_empty() {
            builder.push_record(vec![format!("Mutated Objects: ")]);
            for oref in self.mutated() {
                builder.push_record(vec![owned_objref_string(oref)]);
            }
        }

        if !self.shared_objects().is_empty() {
            builder.push_record(vec![format!("Shared Objects: ")]);
            for oref in self.shared_objects() {
                builder.push_record(vec![objref_string(oref)]);
            }
        }

        if !self.deleted().is_empty() {
            builder.push_record(vec![format!("Deleted Objects: ")]);

            for oref in self.deleted() {
                builder.push_record(vec![objref_string(oref)]);
            }
        }

        if !self.wrapped().is_empty() {
            builder.push_record(vec![format!("Wrapped Objects: ")]);

            for oref in self.wrapped() {
                builder.push_record(vec![objref_string(oref)]);
            }
        }

        if !self.unwrapped().is_empty() {
            builder.push_record(vec![format!("Unwrapped Objects: ")]);
            for oref in self.unwrapped() {
                builder.push_record(vec![owned_objref_string(oref)]);
            }
        }

        builder.push_record(vec![format!(
            "Gas Object: \n{}",
            owned_objref_string(self.gas_object())
        )]);

        let gas_cost_summary = self.gas_cost_summary();
        builder.push_record(vec![format!(
            "Gas Cost Summary:\n   \
             Storage Cost: {} MIST\n   \
             Computation Cost: {} MIST\n   \
             Storage Rebate: {} MIST\n   \
             Non-refundable Storage Fee: {} MIST",
            gas_cost_summary.storage_cost,
            gas_cost_summary.computation_cost,
            gas_cost_summary.storage_rebate,
            gas_cost_summary.non_refundable_storage_fee,
        )]);

        let dependencies = self.dependencies();
        if !dependencies.is_empty() {
            builder.push_record(vec![format!("\nTransaction Dependencies:")]);
            for dependency in dependencies {
                builder.push_record(vec![format!("   {}", dependency)]);
            }
        }

        let mut table = builder.build();
        table.with(TablePanel::header("Transaction Effects"));
        table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
            1,
            TableStyle::modern().get_horizontal(),
        )]));
        write!(f, "{}", table)
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DryRunTransactionBlockResponse {
    pub effects: SuiTransactionBlockEffects,
    pub events: SuiTransactionBlockEvents,
    pub object_changes: Vec<ObjectChange>,
    pub balance_changes: Vec<BalanceChange>,
    pub input: SuiTransactionBlockData,
}

#[derive(Eq, PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename = "TransactionBlockEvents", transparent)]
pub struct SuiTransactionBlockEvents {
    pub data: Vec<SuiEvent>,
}

impl Display for SuiTransactionBlockEvents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.data.is_empty() {
            writeln!(f, "╭─────────────────────────────╮")?;
            writeln!(f, "│ No transaction block events │")?;
            writeln!(f, "╰─────────────────────────────╯")
        } else {
            let mut builder = TableBuilder::default();

            for event in &self.data {
                builder.push_record(vec![format!("{}", event)]);
            }

            let mut table = builder.build();
            table.with(TablePanel::header("Transaction Block Events"));
            table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
                1,
                TableStyle::modern().get_horizontal(),
            )]));
            write!(f, "{}", table)
        }
    }
}

/// Additional rguments supplied to dev inspect beyond what is allowed in today's API.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename = "DevInspectArgs", rename_all = "camelCase")]
pub struct DevInspectArgs {
    /// The sponsor of the gas for the transaction, might be different from the sender.
    pub gas_sponsor: Option<SuiAddress>,
    /// The gas budget for the transaction.
    pub gas_budget: Option<BigInt<u64>>,
    /// The gas objects used to pay for the transaction.
    pub gas_objects: Option<Vec<ObjectRef>>,
    /// Whether to skip transaction checks for the transaction.
    pub skip_checks: Option<bool>,
    /// Whether to return the raw transaction data and effects.
    pub show_raw_txn_data_and_effects: Option<bool>,
}

/// The response from processing a dev inspect transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "DevInspectResults", rename_all = "camelCase")]
pub struct DevInspectResults {
    /// Summary of effects that likely would be generated if the transaction is actually run.
    /// Note however, that not all dev-inspect transactions are actually usable as transactions so
    /// it might not be possible actually generate these effects from a normal transaction.
    pub effects: SuiTransactionBlockEffects,
    /// Events that likely would be generated if the transaction is actually run.
    pub events: SuiTransactionBlockEvents,
    /// Execution results (including return values) from executing the transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<SuiExecutionResult>>,
    /// Execution error from executing the transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The raw transaction data that was dev inspected.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_txn_data: Vec<u8>,
    /// The raw effects of the transaction that was dev inspected.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_effects: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "SuiExecutionResult", rename_all = "camelCase")]
pub struct SuiExecutionResult {
    /// The value of any arguments that were mutably borrowed.
    /// Non-mut borrowed values are not included
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutable_reference_outputs: Vec<(/* argument */ SuiArgument, Vec<u8>, SuiTypeTag)>,
    /// The return values from the transaction
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub return_values: Vec<(Vec<u8>, SuiTypeTag)>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum SuiTransactionBlockBuilderMode {
    /// Regular Sui Transactions that are committed on chain
    Commit,
    /// Simulated transaction that allows calling any Move function with
    /// arbitrary values.
    DevInspect,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "ExecutionStatus", rename_all = "camelCase", tag = "status")]
pub enum SuiExecutionStatus {
    // Gas used in the success case.
    Success,
    // Gas used in the failed case, and the error.
    Failure { error: String },
}

impl Display for SuiExecutionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure { error } => write!(f, "failure due to {error}"),
        }
    }
}

impl SuiExecutionStatus {
    const MOVE_ABORT_PATTERN: &str = r#"MoveAbort\(MoveLocation \{ module: ModuleId \{ address: ([[:alnum:]]+), name: Identifier\("([[:word:]]+)"\) \}, function: (\d+), instruction: (\d+), function_name: Some\("([[:word:]]+)"\) \}, (\d+)\)"#;

    pub fn is_ok(&self) -> bool {
        matches!(self, SuiExecutionStatus::Success { .. })
    }

    pub fn is_err(&self) -> bool {
        matches!(self, SuiExecutionStatus::Failure { .. })
    }

    /// If the error is a [`MoveAbort`], try extracting it.
    ///
    /// [`MoveAbort`]: af_sui_types::ExecutionError::MoveAbort
    pub fn as_move_abort(&self) -> Option<(MoveLocation, u64)> {
        let Self::Failure { error } = self else {
            return None;
        };
        let re = regex::Regex::new(Self::MOVE_ABORT_PATTERN).expect("Tested below");

        let matches = re.captures(error)?;

        // NOTE: prepend with "0x" so that parsing to an address works
        let address = "0x".to_owned() + matches.get(1)?.as_str();
        Some((
            MoveLocation {
                package: address.parse().ok()?,
                module: matches.get(2)?.as_str().parse().ok()?,
                function: matches.get(3)?.as_str().parse().ok()?,
                instruction: matches.get(4)?.as_str().parse().ok()?,
                function_name: Some(matches.get(5)?.as_str().parse().ok()?),
            },
            matches.get(6)?.as_str().parse().ok()?,
        ))
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename = "GasData", rename_all = "camelCase")]
pub struct SuiGasData {
    pub payment: Vec<SuiObjectRef>,
    pub owner: SuiAddress,
    #[serde_as(as = "BigInt<u64>")]
    pub price: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub budget: u64,
}

impl Display for SuiGasData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Gas Owner: {}", self.owner)?;
        writeln!(f, "Gas Budget: {} MIST", self.budget)?;
        writeln!(f, "Gas Price: {} MIST", self.price)?;
        writeln!(f, "Gas Payment:")?;
        for payment in &self.payment {
            write!(f, "{} ", objref_string(payment))?;
        }
        writeln!(f)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[enum_dispatch(SuiTransactionBlockDataAPI)]
#[serde(
    rename = "TransactionBlockData",
    rename_all = "camelCase",
    tag = "messageVersion"
)]
pub enum SuiTransactionBlockData {
    V1(SuiTransactionBlockDataV1),
}

#[enum_dispatch]
pub trait SuiTransactionBlockDataAPI {
    fn transaction(&self) -> &SuiTransactionBlockKind;
    fn sender(&self) -> &SuiAddress;
    fn gas_data(&self) -> &SuiGasData;
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename = "TransactionBlockDataV1", rename_all = "camelCase")]
pub struct SuiTransactionBlockDataV1 {
    pub transaction: SuiTransactionBlockKind,
    pub sender: SuiAddress,
    pub gas_data: SuiGasData,
}

impl SuiTransactionBlockDataAPI for SuiTransactionBlockDataV1 {
    fn transaction(&self) -> &SuiTransactionBlockKind {
        &self.transaction
    }
    fn sender(&self) -> &SuiAddress {
        &self.sender
    }
    fn gas_data(&self) -> &SuiGasData {
        &self.gas_data
    }
}

impl SuiTransactionBlockData {
    pub fn move_calls(&self) -> Vec<&SuiProgrammableMoveCall> {
        match self {
            Self::V1(data) => match &data.transaction {
                SuiTransactionBlockKind::ProgrammableTransaction(pt) => pt
                    .commands
                    .iter()
                    .filter_map(|command| match command {
                        SuiCommand::MoveCall(c) => Some(&**c),
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            },
        }
    }
}

impl Display for SuiTransactionBlockData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::V1(data) => {
                writeln!(f, "Sender: {}", data.sender)?;
                writeln!(f, "{}", self.gas_data())?;
                writeln!(f, "{}", data.transaction)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename = "TransactionBlock", rename_all = "camelCase")]
pub struct SuiTransactionBlock {
    pub data: SuiTransactionBlockData,
    pub tx_signatures: Vec<UserSignature>,
}

impl Display for SuiTransactionBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut builder = TableBuilder::default();

        builder.push_record(vec![format!("{}", self.data)]);
        builder.push_record(vec![format!("Signatures:")]);
        for tx_sig in &self.tx_signatures {
            builder.push_record(vec![format!("   {}\n", tx_sig.to_base64())]);
        }

        let mut table = builder.build();
        table.with(TablePanel::header("Transaction Data"));
        table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
            1,
            TableStyle::modern().get_horizontal(),
        )]));
        write!(f, "{}", table)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiGenesisTransaction {
    pub objects: Vec<ObjectId>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiConsensusCommitPrologue {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiConsensusCommitPrologueV2 {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: ConsensusCommitDigest,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiConsensusCommitPrologueV3 {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,
    #[serde_as(as = "Option<BigInt<u64>>")]
    pub sub_dag_index: Option<u64>,
    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: ConsensusCommitDigest,
    pub consensus_determined_version_assignments: ConsensusDeterminedVersionAssignments,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiAuthenticatorStateUpdate {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,

    pub new_active_jwks: Vec<SuiActiveJwk>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiRandomnessStateUpdate {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,

    #[serde_as(as = "BigInt<u64>")]
    pub randomness_round: u64,
    pub random_bytes: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiEndOfEpochTransaction {
    pub transactions: Vec<SuiEndOfEpochTransactionKind>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum SuiEndOfEpochTransactionKind {
    ChangeEpoch(SuiChangeEpoch),
    AuthenticatorStateCreate,
    AuthenticatorStateExpire(SuiAuthenticatorStateExpire),
    RandomnessStateCreate,
    CoinDenyListStateCreate,
    BridgeStateCreate(CheckpointDigest),
    BridgeCommitteeUpdate(Version),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiAuthenticatorStateExpire {
    #[serde_as(as = "BigInt<u64>")]
    pub min_epoch: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiActiveJwk {
    pub jwk_id: SuiJwkId,
    pub jwk: SuiJWK,

    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiJwkId {
    pub iss: String,
    pub kid: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiJWK {
    pub kty: String,
    pub e: String,
    pub n: String,
    pub alg: String,
}

#[serde_as]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename = "InputObjectKind")]
pub enum SuiInputObjectKind {
    // A Move package, must be immutable.
    MovePackage(ObjectId),
    // A Move object, either immutable, or owned mutable.
    ImmOrOwnedMoveObject(SuiObjectRef),
    // A Move object that's shared and mutable.
    SharedMoveObject {
        id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        initial_shared_version: Version,
        #[serde(default)]
        mutable: bool,
    },
}

/// A series of commands where the results of one command can be used in future
/// commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiProgrammableTransactionBlock {
    /// Input objects or primitive values
    pub inputs: Vec<SuiCallArg>,
    #[serde(rename = "transactions")]
    /// The transactions to be executed sequentially. A failure in any transaction will
    /// result in the failure of the entire programmable transaction block.
    pub commands: Vec<SuiCommand>,
}

impl Display for SuiProgrammableTransactionBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { inputs, commands } = self;
        writeln!(f, "Inputs: {inputs:?}")?;
        writeln!(f, "Commands: [")?;
        for c in commands {
            writeln!(f, "  {c},")?;
        }
        writeln!(f, "]")
    }
}

/// A single transaction in a programmable transaction block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "SuiTransaction")]
pub enum SuiCommand {
    /// A call to either an entry or a public Move function
    MoveCall(Box<SuiProgrammableMoveCall>),
    /// `(Vec<forall T:key+store. T>, address)`
    /// It sends n-objects to the specified address. These objects must have store
    /// (public transfer) and either the previous owner must be an address or the object must
    /// be newly created.
    TransferObjects(Vec<SuiArgument>, SuiArgument),
    /// `(&mut Coin<T>, Vec<u64>)` -> `Vec<Coin<T>>`
    /// It splits off some amounts into a new coins with those amounts
    SplitCoins(SuiArgument, Vec<SuiArgument>),
    /// `(&mut Coin<T>, Vec<Coin<T>>)`
    /// It merges n-coins into the first coin
    MergeCoins(SuiArgument, Vec<SuiArgument>),
    /// Publishes a Move package. It takes the package bytes and a list of the package's transitive
    /// dependencies to link against on-chain.
    Publish(Vec<ObjectId>),
    /// Upgrades a Move package
    Upgrade(Vec<ObjectId>, ObjectId, SuiArgument),
    /// `forall T: Vec<T> -> vector<T>`
    /// Given n-values of the same type, it constructs a vector. For non objects or an empty vector,
    /// the type tag must be specified.
    MakeMoveVec(Option<String>, Vec<SuiArgument>),
}

impl Display for SuiCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MoveCall(p) => {
                write!(f, "MoveCall({p})")
            }
            Self::MakeMoveVec(ty_opt, elems) => {
                write!(f, "MakeMoveVec(")?;
                if let Some(ty) = ty_opt {
                    write!(f, "Some{ty}")?;
                } else {
                    write!(f, "None")?;
                }
                write!(f, ",[")?;
                write_sep(f, elems, ",")?;
                write!(f, "])")
            }
            Self::TransferObjects(objs, addr) => {
                write!(f, "TransferObjects([")?;
                write_sep(f, objs, ",")?;
                write!(f, "],{addr})")
            }
            Self::SplitCoins(coin, amounts) => {
                write!(f, "SplitCoins({coin},")?;
                write_sep(f, amounts, ",")?;
                write!(f, ")")
            }
            Self::MergeCoins(target, coins) => {
                write!(f, "MergeCoins({target},")?;
                write_sep(f, coins, ",")?;
                write!(f, ")")
            }
            Self::Publish(deps) => {
                write!(f, "Publish(<modules>,")?;
                write_sep(f, deps, ",")?;
                write!(f, ")")
            }
            Self::Upgrade(deps, current_package_id, ticket) => {
                write!(f, "Upgrade(<modules>, {ticket},")?;
                write_sep(f, deps, ",")?;
                write!(f, ", {current_package_id}")?;
                write!(f, ")")
            }
        }
    }
}

/// An argument to a transaction in a programmable transaction block
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuiArgument {
    /// The gas coin. The gas coin can only be used by-ref, except for with
    /// `TransferObjects`, which can use it by-value.
    GasCoin,
    /// One of the input objects or primitive values (from
    /// `ProgrammableTransactionBlock` inputs)
    Input(u16),
    /// The result of another transaction (from `ProgrammableTransactionBlock` transactions)
    Result(u16),
    /// Like a `Result` but it accesses a nested result. Currently, the only usage
    /// of this is to access a value from a Move call with multiple return values.
    NestedResult(u16, u16),
}

impl Display for SuiArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GasCoin => write!(f, "GasCoin"),
            Self::Input(i) => write!(f, "Input({i})"),
            Self::Result(i) => write!(f, "Result({i})"),
            Self::NestedResult(i, j) => write!(f, "NestedResult({i},{j})"),
        }
    }
}

/// The transaction for calling a Move function, either an entry function or a public
/// function (which cannot return references).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiProgrammableMoveCall {
    /// The package containing the module and function.
    pub package: ObjectId,
    /// The specific module in the package containing the function.
    pub module: String,
    /// The function to be called.
    pub function: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// The type arguments to the function.
    pub type_arguments: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// The arguments to the function.
    pub arguments: Vec<SuiArgument>,
}

fn write_sep<T: Display>(
    f: &mut Formatter<'_>,
    items: impl IntoIterator<Item = T>,
    sep: &str,
) -> std::fmt::Result {
    let mut xs = items.into_iter().peekable();
    while let Some(x) = xs.next() {
        write!(f, "{x}")?;
        if xs.peek().is_some() {
            write!(f, "{sep}")?;
        }
    }
    Ok(())
}

impl Display for SuiProgrammableMoveCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            package,
            module,
            function,
            type_arguments,
            arguments,
        } = self;
        write!(f, "{package}::{module}::{function}")?;
        if !type_arguments.is_empty() {
            write!(f, "<")?;
            write_sep(f, type_arguments, ",")?;
            write!(f, ">")?;
        }
        write!(f, "(")?;
        write_sep(f, arguments, ",")?;
        write!(f, ")")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "TypeTag", rename_all = "camelCase")]
pub struct SuiTypeTag(String);

impl SuiTypeTag {
    pub fn new(tag: String) -> Self {
        Self(tag)
    }
}

impl TryInto<TypeTag> for SuiTypeTag {
    type Error = <TypeTag as FromStr>::Err;
    fn try_into(self) -> Result<TypeTag, Self::Error> {
        self.0.parse()
    }
}

impl From<TypeTag> for SuiTypeTag {
    fn from(tag: TypeTag) -> Self {
        Self(format!("{}", tag))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RPCTransactionRequestParams {
    TransferObjectRequestParams(TransferObjectParams),
    MoveCallRequestParams(MoveCallParams),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferObjectParams {
    pub recipient: SuiAddress,
    pub object_id: ObjectId,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveCallParams {
    pub package_object_id: ObjectId,
    pub module: String,
    pub function: String,
    #[serde(default)]
    pub type_arguments: Vec<SuiTypeTag>,
    pub arguments: Vec<serde_json::Value>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionBlockBytes {
    /// BCS serialized transaction data bytes without its type tag, as base-64 encoded string.
    pub tx_bytes: String,
    /// the gas objects to be used
    pub gas: Vec<SuiObjectRef>,
    /// objects to be used in this transaction
    pub input_objects: Vec<SuiInputObjectKind>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "OwnedObjectRef")]
pub struct OwnedObjectRef {
    pub owner: Owner,
    pub reference: SuiObjectRef,
}

impl OwnedObjectRef {
    pub fn object_id(&self) -> ObjectId {
        self.reference.object_id
    }
    pub fn version(&self) -> Version {
        self.reference.version
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SuiCallArg {
    // Needs to become an Object Ref or Object ID, depending on object type
    Object(SuiObjectArg),
    // pure value, bcs encoded
    Pure(SuiPureValue),
}

impl SuiCallArg {
    pub fn pure(&self) -> Option<&serde_json::Value> {
        match self {
            SuiCallArg::Pure(v) => Some(&v.value),
            _ => None,
        }
    }

    pub fn object(&self) -> Option<&ObjectId> {
        match self {
            SuiCallArg::Object(SuiObjectArg::SharedObject { object_id, .. })
            | SuiCallArg::Object(SuiObjectArg::ImmOrOwnedObject { object_id, .. })
            | SuiCallArg::Object(SuiObjectArg::Receiving { object_id, .. }) => Some(object_id),
            _ => None,
        }
    }
}

#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiPureValue {
    // #[serde_as(as = "Option<AsSuiTypeTag>")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    value_type: Option<TypeTag>,
    value: serde_json::Value,
}

impl SuiPureValue {
    pub fn value(&self) -> serde_json::Value {
        self.value.clone()
    }

    pub fn value_type(&self) -> Option<TypeTag> {
        self.value_type.clone()
    }
}

#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "objectType", rename_all = "camelCase")]
pub enum SuiObjectArg {
    // A Move object, either immutable, or owned mutable.
    #[serde(rename_all = "camelCase")]
    ImmOrOwnedObject {
        object_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        version: Version,
        digest: ObjectDigest,
    },
    // A Move object that's shared.
    // SharedObject::mutable controls whether caller asks for a mutable reference to shared object.
    #[serde(rename_all = "camelCase")]
    SharedObject {
        object_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        initial_shared_version: Version,
        mutable: bool,
    },
    // A reference to a Move object that's going to be received in the transaction.
    #[serde(rename_all = "camelCase")]
    Receiving {
        object_id: ObjectId,
        #[serde_as(as = "BigInt<u64>")]
        version: Version,
        digest: ObjectDigest,
    },
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TransactionFilter {
    /// CURRENTLY NOT SUPPORTED. Query by checkpoint.
    Checkpoint(#[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")] Version),
    /// Query by move function.
    MoveFunction {
        package: ObjectId,
        module: Option<String>,
        function: Option<String>,
    },
    /// Query by input object.
    InputObject(ObjectId),
    /// Query by changed object, including created, mutated and unwrapped objects.
    ChangedObject(ObjectId),
    /// Query for transactions that touch this object.
    AffectedObject(ObjectId),
    /// Query by sender address.
    FromAddress(SuiAddress),
    /// Query by recipient address.
    ToAddress(SuiAddress),
    /// Query by sender and recipient address.
    FromAndToAddress { from: SuiAddress, to: SuiAddress },
    /// CURRENTLY NOT SUPPORTED. Query txs that have a given address as sender or recipient.
    FromOrToAddress { addr: SuiAddress },
    /// Query by transaction kind
    TransactionKind(String),
    /// Query transactions of any given kind in the input.
    TransactionKindIn(Vec<String>),
}

#[cfg(test)]
mod tests {
    use af_sui_types::IdentStr;
    use color_eyre::Result;
    use itertools::Itertools as _;

    use super::*;

    const MOVE_ABORT_ERRORS: [&str; 3] = [
        r#"MoveAbort(MoveLocation { module: ModuleId { address: fd6f306bb2f8dce24dd3d4a9bdc51a46e7c932b15007d73ac0cfb38c15de0fea, name: Identifier("market") }, function: 1, instruction: 60, function_name: Some("try_update_funding") }, 1001)"#,
        r#"MoveAbort(MoveLocation { module: ModuleId { address: 241537381737a40df6838bc395fb64f04ff604513c18a2ac3308ac810c805fa6, name: Identifier("oracle") }, function: 23, instruction: 42, function_name: Some("update_price_feed_inner") }, 4)"#,
        r#"MoveAbort(MoveLocation { module: ModuleId { address: 72a8715095cdc8442b4316f78802d7aefa2e6f0c3c6fac256ce81554034b0d4b, name: Identifier("clearing_house") }, function: 53, instruction: 32, function_name: Some("settled_liquidated_position") }, 2001) in command 3"#,
    ];

    #[test]
    fn move_abort_regex_is_valid() -> Result<()> {
        regex::Regex::new(SuiExecutionStatus::MOVE_ABORT_PATTERN)?;
        Ok(())
    }

    #[test]
    fn move_abort_extracts() -> Result<()> {
        let expected = [
            (
                MoveLocation {
                    package: "0xfd6f306bb2f8dce24dd3d4a9bdc51a46e7c932b15007d73ac0cfb38c15de0fea"
                        .parse()?,
                    module: IdentStr::cast("market").to_owned(),
                    function: 1,
                    instruction: 60,
                    function_name: Some("try_update_funding".parse()?),
                },
                1001,
            ),
            (
                MoveLocation {
                    package: "0x241537381737a40df6838bc395fb64f04ff604513c18a2ac3308ac810c805fa6"
                        .parse()?,
                    module: IdentStr::cast("oracle").to_owned(),
                    function: 23,
                    instruction: 42,
                    function_name: Some("update_price_feed_inner".parse()?),
                },
                4,
            ),
            (
                MoveLocation {
                    package: "0x72a8715095cdc8442b4316f78802d7aefa2e6f0c3c6fac256ce81554034b0d4b"
                        .parse()?,
                    module: IdentStr::cast("clearing_house").to_owned(),
                    function: 53,
                    instruction: 32,
                    function_name: Some("settled_liquidated_position".parse()?),
                },
                2001,
            ),
        ];

        let errors = MOVE_ABORT_ERRORS
            .into_iter()
            .map(|msg| SuiExecutionStatus::Failure { error: msg.into() });

        for (error, expect) in errors.into_iter().zip_eq(expected) {
            assert_eq!(error.as_move_abort(), Some(expect));
        }

        Ok(())
    }
}
