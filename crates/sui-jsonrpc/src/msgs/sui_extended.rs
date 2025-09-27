// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use af_sui_types::{Address as SuiAddress, CheckpointSequenceNumber, Identifier, ObjectId};
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::{DisplayFromStr, IfIsHumanReadable, serde_as};
use sui_sdk_types::EpochId;

use super::Page;
use crate::serde::BigInt;

pub type EpochPage = Page<EpochInfo, BigInt<u64>>;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EpochInfo {
    /// epoch number
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: EpochId,
    /// list of validators included in epoch
    pub validators: Vec<SuiValidatorSummary>,
    /// count of tx in epoch
    #[serde_as(as = "BigInt<u64>")]
    pub epoch_total_transactions: u64,
    /// first, last checkpoint sequence numbers
    #[serde_as(as = "BigInt<u64>")]
    pub first_checkpoint_id: CheckpointSequenceNumber,
    #[serde_as(as = "BigInt<u64>")]
    pub epoch_start_timestamp: u64,
    pub end_of_epoch_info: Option<EndOfEpochInfo>,
    pub reference_gas_price: Option<u64>,
}

/// This is the JSON-RPC type for the SUI validator. It flattens all inner structures
/// to top-level fields so that they are decoupled from the internal definitions.
///
/// Originally from `sui_types::sui_system_state::sui_system_state_summary`.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidatorSummary {
    // Metadata
    pub sui_address: SuiAddress,
    #[serde_as(as = "Base64")]
    pub protocol_pubkey_bytes: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub network_pubkey_bytes: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub worker_pubkey_bytes: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub proof_of_possession_bytes: Vec<u8>,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub project_url: String,
    pub net_address: String,
    pub p2p_address: String,
    pub primary_address: String,
    pub worker_address: String,
    #[serde_as(as = "Option<Base64>")]
    pub next_epoch_protocol_pubkey_bytes: Option<Vec<u8>>,
    #[serde_as(as = "Option<Base64>")]
    pub next_epoch_proof_of_possession: Option<Vec<u8>>,
    #[serde_as(as = "Option<Base64>")]
    pub next_epoch_network_pubkey_bytes: Option<Vec<u8>>,
    #[serde_as(as = "Option<Base64>")]
    pub next_epoch_worker_pubkey_bytes: Option<Vec<u8>>,
    pub next_epoch_net_address: Option<String>,
    pub next_epoch_p2p_address: Option<String>,
    pub next_epoch_primary_address: Option<String>,
    pub next_epoch_worker_address: Option<String>,

    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub voting_power: u64,
    pub operation_cap_id: ObjectId,
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub gas_price: u64,
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub commission_rate: u64,
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub next_epoch_stake: u64,
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub next_epoch_gas_price: u64,
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub next_epoch_commission_rate: u64,

    // Staking pool information
    /// ID of the staking pool object.
    pub staking_pool_id: ObjectId,
    /// The epoch at which this pool became active.
    #[serde_as(as = "Option<IfIsHumanReadable<BigInt<u64>, _>>")]
    pub staking_pool_activation_epoch: Option<u64>,
    /// The epoch at which this staking pool ceased to be active. `None` = {pre-active, active},
    #[serde_as(as = "Option<IfIsHumanReadable<BigInt<u64>, _>>")]
    pub staking_pool_deactivation_epoch: Option<u64>,
    /// The total number of SUI tokens in this pool.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub staking_pool_sui_balance: u64,
    /// The epoch stake rewards will be added here at the end of each epoch.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub rewards_pool: u64,
    /// Total number of pool tokens issued by the pool.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub pool_token_balance: u64,
    /// Pending stake amount for this epoch.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub pending_stake: u64,
    /// Pending stake withdrawn during the current epoch, emptied at epoch boundaries.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub pending_total_sui_withdraw: u64,
    /// Pending pool token withdrawn during the current epoch, emptied at epoch boundaries.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub pending_pool_token_withdraw: u64,
    /// ID of the exchange rate table object.
    pub exchange_rates_id: ObjectId,
    /// Number of exchange rates in the table.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub exchange_rates_size: u64,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EndOfEpochInfo {
    #[serde_as(as = "BigInt<u64>")]
    pub last_checkpoint_id: CheckpointSequenceNumber,
    #[serde_as(as = "BigInt<u64>")]
    pub epoch_end_timestamp: u64,
    /// existing fields from `SystemEpochInfo`
    #[serde_as(as = "BigInt<u64>")]
    pub protocol_version: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub reference_gas_price: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub total_stake: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub storage_fund_reinvestment: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub storage_charge: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub storage_rebate: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub storage_fund_balance: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub stake_subsidy_amount: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub total_gas_fees: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub total_stake_rewards_distributed: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub leftover_storage_fund_inflow: u64,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MoveFunctionName {
    pub package: ObjectId,
    #[serde_as(as = "DisplayFromStr")]
    pub module: Identifier,
    #[serde_as(as = "DisplayFromStr")]
    pub function: Identifier,
}
