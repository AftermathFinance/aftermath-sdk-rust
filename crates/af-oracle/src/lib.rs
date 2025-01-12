#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Move types for Aftermath's `AfOracle` package

use af_utilities::types::IFixed;
use sui_framework_sdk::object::{ID, UID};

pub mod errors;
pub mod event_ext;
pub mod event_instance;
#[cfg(feature = "graphql")]
pub mod graphql;

af_sui_pkg_sdk::sui_pkg_sdk!(af_oracle {
    module oracle {
        // =========================================================================
        //  Objects
        // =========================================================================

        /// Capability object required to create/remove price feeds.
        ///
        /// Created as a single-writer object, unique.
        struct AuthorityCap has key, store {
            id: UID,
        }

        /// Object that stores all the price feeds for this aggregation.
        struct PriceFeedStorage has key, store {
            id: UID,
            /// Symbol for this price feed storage
            symbol: String,
            /// Amount of decimals for this price feed storage.
            decimals: u64,
            /// List of source wrapper ids having a feed in this storage.
            /// Each ID is used as key for the dynamic field containing the price feed
            source_wrapper_ids: vector<ID>,
        }

        /// Object that identifies a price feed for a single source.
        struct PriceFeed has key, store {
            id: UID,
            /// Price value.
            price: IFixed,
            /// Price timestamp.
            timestamp: u64,
            /// Allowed gap between current
            /// and provided timestamp.
            time_tolerance: u64,
        }

        /// Signals a package's capability to act as a price feed source.
        struct SourceCap has store {}
    }

    module keys {
        /// Key type for accessing price feed for particular source.
        struct PriceFeedForSource has copy, drop, store {
            source_wrapper_id: ID,
        }

        /// For attaching a capability to an app.
        ///
        /// This ensures only this package has access to this dynamic field since the key can only be
        /// constructed here.
        struct Authorization has copy, drop, store {}
    }

    module events {
        struct CreatedPriceFeedStorage has copy, drop {
            price_feed_storage_id: ID,
            symbol: String,
            decimals: u64
        }

        struct AddedAuthorization has copy, drop {
            source_wrapper_id: ID,
        }

        struct RemovedAuthorization has copy, drop {
            source_wrapper_id: ID,
        }

        struct CreatedPriceFeed has copy, drop {
            price_feed_storage_id: ID,
            source_wrapper_id: ID,
            price: IFixed,
            timestamp: u64,
            time_tolerance: u64,
        }

        struct RemovedPriceFeed has copy, drop {
            price_feed_storage_id: ID,
            source_wrapper_id: ID,
        }

        struct UpdatedPriceFeed has copy, drop {
            price_feed_storage_id: ID,
            source_wrapper_id: ID,
            old_price: IFixed,
            old_timestamp: u64,
            new_price: IFixed,
            new_timestamp: u64,
        }

        struct UpdatedPriceFeedTimeTolerance has copy, drop {
            price_feed_storage_id: ID,
            source_wrapper_id: ID,
            old_time_tolerance: u64,
            new_time_tolerance: u64,
        }
    }
});
