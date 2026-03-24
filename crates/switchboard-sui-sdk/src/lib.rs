//! Sdk for Switchboard's Sui package.

use af_sui_pkg_sdk::sui_pkg_sdk;
use move_stdlib_sdk::type_name::TypeName;
use sui_framework_sdk::object::{ID, UID};
use sui_framework_sdk::table::Table;

#[cfg(feature = "ptb")]
pub mod update;

sui_pkg_sdk!(switchboard {
    module aggregator {

        public struct CurrentResult has copy, drop, store {
            result: decimal::Decimal,
            timestamp_ms: u64,
            min_timestamp_ms: u64,
            max_timestamp_ms: u64,
            min_result: decimal::Decimal,
            max_result: decimal::Decimal,
            stdev: decimal::Decimal,
            range: decimal::Decimal,
            mean: decimal::Decimal,
        }

        public struct Update has copy, drop, store {
            result: decimal::Decimal,
            timestamp_ms: u64,
            oracle: ID,
        }

        public struct UpdateState has store {
            results: vector<Update>,
            curr_idx: u64,
        }

        public struct Aggregator has key {
            id: UID,

            // The queue this aggregator is associated with
            queue: ID,

            // The time this aggregator was created
            created_at_ms: u64,

            // -- Configs --

            // The name of the aggregator
            name: String,

            // The address of the authority that created this aggregator
            authority: address,

            // The hash of the feed this aggregator is associated with
            feed_hash: vector<u8>,

            // The minimum number of updates to consider the result valid
            min_sample_size: u64,

            // The maximum number of samples to consider the an update valid
            max_staleness_seconds: u64,

            // The maximum variance between jobs required for a result to be computed
            max_variance: u64,

            // Minimum number of job successes required to compute a valid update
            min_responses: u32,


            // -- State --

            // The current result of the aggregator
            current_result: CurrentResult,

            // The state of the updates
            update_state: UpdateState,

            // version
            version: u8,
        }
    }

    module oracle {
        public struct Attestation has copy, store, drop {
            guardian_id: ID,
            secp256k1_key: vector<u8>,
            timestamp_ms: u64,
        }

        public struct Oracle has key {
            id: UID,
            oracle_key: vector<u8>,
            queue: ID,
            queue_key: vector<u8>,
            expiration_time_ms: u64,
            mr_enclave: vector<u8>,
            secp256k1_key: vector<u8>,
            valid_attestations: vector<Attestation>,
            version: u8,
        }
    }

    module queue {
        public struct ExistingOracle has copy, drop, store {
            oracle_id: ID,
            oracle_key: vector<u8>,
        }

        public struct Queue has key {
            id: UID,
            queue_key: vector<u8>,
            authority: address,
            name: String,
            fee: u64,
            fee_recipient: address,
            min_attestations: u64,
            oracle_validity_length_ms: u64,
            last_queue_override_ms: u64,
            guardian_queue_id: ID,

            // to ensure that oracles are only mapped once (oracle pubkeys)
            existing_oracles: Table<vector<u8>,ExistingOracle>,
            fee_types: vector<TypeName>,
            version: u8,
        }
    }

    module decimal {
        public struct Decimal has copy, drop, store { value: u128, neg: bool }
    }

    module hash {
        public struct Hasher has drop, copy {
            buffer: vector<u8>,
        }
    }

    // Events
    module aggregator_delete_action {
        public struct AggregatorDeleted has copy, drop {
            aggregator_id: ID,
        }
    }

    module aggregator_init_action {
        public struct AggregatorCreated has copy, drop {
            aggregator_id: ID,
            name: String,
        }
    }

    module aggregator_set_authority_action {
        public struct AggregatorAuthorityUpdated has copy, drop {
            aggregator_id: ID,
            existing_authority: address,
            new_authority: address,
        }
    }

    module aggregator_set_configs_action {
        public struct AggregatorConfigsUpdated has copy, drop {
            aggregator_id: ID,
            feed_hash: vector<u8>,
            min_sample_size: u64,
            max_staleness_seconds: u64,
            max_variance: u64,
            min_responses: u32,
        }
    }

    module aggregator_submit_result_action {
        public struct AggregatorUpdated has copy, drop {
            aggregator_id: ID,
            oracle_id: ID,
            value: decimal::Decimal,
            timestamp_ms: u64,
        }
    }

    module oracle_attest_action {
        public struct AttestationCreated has copy, drop {
            oracle_id: ID,
            guardian_id: ID,
            secp256k1_key: vector<u8>,
            timestamp_ms: u64,
        }

        public struct AttestationResolved has copy, drop {
            oracle_id: ID,
            secp256k1_key: vector<u8>,
            timestamp_ms: u64,
        }
    }

    module oracle_init_action {
        public struct OracleCreated has copy, drop {
            oracle_id: ID,
            queue_id: ID,
            oracle_key: vector<u8>,
        }
    }

    module guardian_queue_init_action {
        public struct GuardianQueueCreated has copy, drop {
            queue_id: ID,
            queue_key: vector<u8>,
        }
    }

    module oracle_queue_init_action {
        public struct OracleQueueCreated has copy, drop {
            queue_id: ID,
            guardian_queue_id: ID,
            queue_key: vector<u8>,
        }
    }

    module queue_add_fee_coin_action {
        public struct QueueFeeTypeAdded has copy, drop {
            queue_id: ID,
            fee_type: TypeName,
        }
    }

    module queue_override_oracle_action {
        public struct QueueOracleOverride has copy, drop {
            queue_id: ID,
            oracle_id: ID,
            secp256k1_key: vector<u8>,
            mr_enclave: vector<u8>,
            expiration_time_ms: u64,
        }
    }

    module queue_remove_fee_coin_action {
        public struct QueueFeeTypeRemoved has copy, drop {
            queue_id: ID,
            fee_type: TypeName,
        }
    }

    module queue_set_authority_action {
        public struct QueueAuthorityUpdated has copy, drop {
            queue_id: ID,
            existing_authority: address,
            new_authority: address,
        }
    }

    module queue_set_configs_action {
        public struct QueueConfigsUpdated has copy, drop {
            queue_id: ID,
            name: String,
            fee: u64,
            fee_recipient: address,
            min_attestations: u64,
            oracle_validity_length_ms: u64,
        }
    }

    module set_guardian_queue_id_action {
        public struct GuardianQueueIdSet has copy, drop {
            old_guardian_queue_id: ID,
            guardian_queue_id: ID,
        }
    }

    module set_oracle_queue_id_action {
        public struct OracleQueueIdSet has copy, drop {
            old_oracle_queue_id: ID,
            oracle_queue_id: ID,
        }
    }

    module set_package_id_action {
        public struct OnDemandPackageIdSet has copy, drop {
            old_on_demand_package_id: ID,
            on_demand_package_id: ID,
        }
    }
});
