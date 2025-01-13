// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use af_sui_types::{Address as SuiAddress, EpochId, ObjectId};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, IfIsHumanReadable};
use sui_sdk_types::Bls12381PublicKey;

use super::SuiValidatorSummary;
use crate::serde::BigInt;

/// Originally `sui_types::committee::StakeUnit`.
pub type StakeUnit = u64;

/// This is the JSON-RPC type for the SUI system state object.
///
/// It flattens all fields to make them top-level fields such that it as minimum
/// dependencies to the internal data structures of the SUI system state type.
///
/// Originally `sui_types::sui_system_state::sui_system_state_summary::SuiSystemStateSummary`.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuiSystemStateSummary {
    /// The current epoch ID, starting from 0.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub epoch: u64,
    /// The current protocol version, starting from 1.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub protocol_version: u64,
    /// The current version of the system state data structure type.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub system_state_version: u64,
    /// The storage rebates of all the objects on-chain stored in the storage fund.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub storage_fund_total_object_storage_rebates: u64,
    /// The non-refundable portion of the storage fund coming from storage reinvestment, non-refundable
    /// storage rebates and any leftover staking rewards.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub storage_fund_non_refundable_balance: u64,
    /// The reference gas price for the current epoch.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub reference_gas_price: u64,
    /// Whether the system is running in a downgraded safe mode due to a non-recoverable bug.
    /// This is set whenever we failed to execute advance_epoch, and ended up executing advance_epoch_safe_mode.
    /// It can be reset once we are able to successfully execute advance_epoch.
    pub safe_mode: bool,
    /// Amount of storage rewards accumulated (and not yet distributed) during safe mode.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub safe_mode_storage_rewards: u64,
    /// Amount of computation rewards accumulated (and not yet distributed) during safe mode.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub safe_mode_computation_rewards: u64,
    /// Amount of storage rebates accumulated (and not yet burned) during safe mode.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub safe_mode_storage_rebates: u64,
    /// Amount of non-refundable storage fee accumulated during safe mode.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub safe_mode_non_refundable_storage_fee: u64,
    /// Unix timestamp of the current epoch start
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub epoch_start_timestamp_ms: u64,

    // System parameters
    /// The duration of an epoch, in milliseconds.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub epoch_duration_ms: u64,

    /// The starting epoch in which stake subsidies start being paid out
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub stake_subsidy_start_epoch: u64,

    /// Maximum number of active validators at any moment.
    /// We do not allow the number of validators in any epoch to go above this.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub max_validator_count: u64,

    /// Lower-bound on the amount of stake required to become a validator.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub min_validator_joining_stake: u64,

    /// Validators with stake amount below `validator_low_stake_threshold` are considered to
    /// have low stake and will be escorted out of the validator set after being below this
    /// threshold for more than `validator_low_stake_grace_period` number of epochs.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub validator_low_stake_threshold: u64,

    /// Validators with stake below `validator_very_low_stake_threshold` will be removed
    /// immediately at epoch change, no grace period.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub validator_very_low_stake_threshold: u64,

    /// A validator can have stake below `validator_low_stake_threshold`
    /// for this many epochs before being kicked out.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub validator_low_stake_grace_period: u64,

    // Stake subsidy information
    /// Balance of SUI set aside for stake subsidies that will be drawn down over time.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub stake_subsidy_balance: u64,
    /// This counter may be different from the current epoch number if
    /// in some epochs we decide to skip the subsidy.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub stake_subsidy_distribution_counter: u64,
    /// The amount of stake subsidy to be drawn down per epoch.
    /// This amount decays and decreases over time.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub stake_subsidy_current_distribution_amount: u64,
    /// Number of distributions to occur before the distribution amount decays.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub stake_subsidy_period_length: u64,
    /// The rate at which the distribution amount decays at the end of each
    /// period. Expressed in basis points.
    pub stake_subsidy_decrease_rate: u16,

    // Validator set
    /// Total amount of stake from all active validators at the beginning of the epoch.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub total_stake: u64,
    /// The list of active validators in the current epoch.
    pub active_validators: Vec<SuiValidatorSummary>,
    /// ID of the object that contains the list of new validators that will join at the end of the epoch.
    pub pending_active_validators_id: ObjectId,
    /// Number of new validators that will join at the end of the epoch.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub pending_active_validators_size: u64,
    /// Removal requests from the validators. Each element is an index
    /// pointing to `active_validators`.
    #[serde_as(as = "Vec<IfIsHumanReadable<BigInt<u64>, _>>")]
    pub pending_removals: Vec<u64>,
    /// ID of the object that maps from staking pool's ID to the sui address of a validator.
    pub staking_pool_mappings_id: ObjectId,
    /// Number of staking pool mappings.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub staking_pool_mappings_size: u64,
    /// ID of the object that maps from a staking pool ID to the inactive validator that has that pool as its staking pool.
    pub inactive_pools_id: ObjectId,
    /// Number of inactive staking pools.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub inactive_pools_size: u64,
    /// ID of the object that stores preactive validators, mapping their addresses to their `Validator` structs.
    pub validator_candidates_id: ObjectId,
    /// Number of preactive validators.
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub validator_candidates_size: u64,
    /// Map storing the number of epochs for which each validator has been below the low stake threshold.
    #[serde_as(as = "Vec<(_, IfIsHumanReadable<BigInt<u64>, _>)>")]
    pub at_risk_validators: Vec<(SuiAddress, u64)>,
    /// A map storing the records of validator reporting each other.
    pub validator_report_records: Vec<(SuiAddress, Vec<SuiAddress>)>,
}

/// RPC representation of the [Committee](https://mystenlabs.github.io/sui/sui_types/committee/struct.Committee.html)
/// type.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "CommitteeInfo")]
pub struct SuiCommittee {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: EpochId,
    #[serde_as(as = "Vec<(_, BigInt<u64>)>")]
    pub validators: Vec<(Bls12381PublicKey, StakeUnit)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DelegatedStake {
    /// Validator's Address.
    pub validator_address: SuiAddress,
    /// Staking pool object id.
    pub staking_pool: ObjectId,
    pub stakes: Vec<Stake>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "status")]
pub enum StakeStatus {
    Pending,
    #[serde(rename_all = "camelCase")]
    Active {
        #[serde_as(as = "BigInt<u64>")]
        estimated_reward: u64,
    },
    Unstaked,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Stake {
    /// ID of the StakedSui receipt object.
    pub staked_sui_id: ObjectId,
    #[serde_as(as = "BigInt<u64>")]
    pub stake_request_epoch: EpochId,
    #[serde_as(as = "BigInt<u64>")]
    pub stake_active_epoch: EpochId,
    #[serde_as(as = "BigInt<u64>")]
    pub principal: u64,
    #[serde(flatten)]
    pub status: StakeStatus,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidatorApys {
    pub apys: Vec<ValidatorApy>,
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: EpochId,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidatorApy {
    pub address: SuiAddress,
    pub apy: f64,
}
