// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use af_sui_types::{CheckpointDigest, EpochId, GasCostSummary, TransactionDigest};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sui_sdk_types::types::{
    CheckpointCommitment,
    CheckpointSequenceNumber,
    CheckpointTimestamp,
    EndOfEpochData,
};

use super::Page;
use crate::serde::{BigInt, GasCostSummaryJson};

pub type CheckpointPage = Page<Checkpoint, BigInt<u64>>;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Checkpoint {
    /// Checkpoint's epoch ID
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: EpochId,
    /// Checkpoint sequence number
    #[serde_as(as = "BigInt<u64>")]
    pub sequence_number: CheckpointSequenceNumber,
    /// Checkpoint digest
    pub digest: CheckpointDigest,
    /// Total number of transactions committed since genesis, including those in this
    /// checkpoint.
    #[serde_as(as = "BigInt<u64>")]
    pub network_total_transactions: u64,
    /// Digest of the previous checkpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_digest: Option<CheckpointDigest>,
    /// The running total gas costs of all transactions included in the current epoch so far
    /// until this checkpoint.
    #[serde_as(as = "serde_with::FromInto<GasCostSummaryJson>")]
    pub epoch_rolling_gas_cost_summary: GasCostSummary,
    /// Timestamp of the checkpoint - number of milliseconds from the Unix epoch
    /// Checkpoint timestamps are monotonic, but not strongly monotonic - subsequent
    /// checkpoints can have same timestamp if they originate from the same underlining consensus commit
    #[serde_as(as = "BigInt<u64>")]
    pub timestamp_ms: CheckpointTimestamp,
    /// Present only on the final checkpoint of the epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_epoch_data: Option<EndOfEpochData>,
    /// Transaction digests
    pub transactions: Vec<TransactionDigest>,

    /// Commitments to checkpoint state
    pub checkpoint_commitments: Vec<CheckpointCommitment>,
    /// Validator Signature
    pub validator_signature: sui_sdk_types::types::Bls12381Signature,
}

#[serde_as]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, derive_more::From)]
#[serde(untagged)]
pub enum CheckpointId {
    SequenceNumber(#[serde_as(as = "BigInt<u64>")] CheckpointSequenceNumber),
    Digest(CheckpointDigest),
}
