#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Sdk for Pyth's Sui package.

use af_sui_pkg_sdk::sui_pkg_sdk;
use sui_framework_sdk::object::UID;
use sui_framework_sdk::package::UpgradeCap;
use wormhole_sui_sdk::consumed_vaas::ConsumedVAAs;
use wormhole_sui_sdk::external_address::ExternalAddress;

#[cfg(feature = "pyth-sdk")]
mod pyth_sdk;
#[cfg(feature = "json-rpc")]
pub mod read;
#[cfg(feature = "ptb")]
pub mod update;

sui_pkg_sdk!(pyth {
    module state {
        /// Capability reflecting that the current build version is used to invoke
        /// state methods.
        struct LatestOnly has drop {}

        struct State has key, store {
            id: UID,
            governance_data_source: data_source::DataSource,
            stale_price_threshold: u64,
            base_update_fee: u64,
            fee_recipient_address: address,
            last_executed_governance_sequence: u64,
            consumed_vaas: ConsumedVAAs,

            // Upgrade capability.
            upgrade_cap: UpgradeCap
        }
    }

    module data_source {
        struct DataSource has copy, drop, store {
            emitter_chain: u64,
            emitter_address: ExternalAddress,
        }
    }

    module price_info {
        /// Sui object version of PriceInfo.
        /// Has a key ability, is unique for each price identifier, and lives in global store.
        struct PriceInfoObject has key, store {
            id: UID,
            price_info: PriceInfo
        }

        /// Copyable and droppable.
        struct PriceInfo has copy, drop, store {
            attestation_time: u64,
            arrival_time: u64,
            price_feed: price_feed::PriceFeed,
        }
    }

    module price_feed {
        /// PriceFeed represents a current aggregate price for a particular product.
        struct PriceFeed has copy, drop, store {
            /// The price identifier
            price_identifier: price_identifier::PriceIdentifier,
            /// The current aggregate price
            price: price::Price,
            /// The current exponentially moving average aggregate price
            ema_price: price::Price,
        }
    }

    module price_identifier {
        struct PriceIdentifier has copy, drop, store {
            bytes: vector<u8>,
        }
    }

    module price {
        /// A price with a degree of uncertainty, represented as a price +- a confidence interval.
        ///
        /// The confidence interval roughly corresponds to the standard error of a normal distribution.
        /// Both the price and confidence are stored in a fixed-point numeric representation,
        /// `x * (10^expo)`, where `expo` is the exponent.
        //
        /// Please refer to the documentation at https://docs.pyth.network/documentation/pythnet-price-feeds/best-practices for how
        /// to how this price safely.
        struct Price has copy, drop, store {
            price: i64::I64,
            /// Confidence interval around the price
            conf: u64,
            /// The exponent
            expo: i64::I64,
            /// Unix timestamp of when this price was computed
            timestamp: u64,
        }
    }

    module i64 {
        /// As Move does not support negative numbers natively, we use our own internal
        /// representation.
        ///
        /// To consume these values, first call `get_is_negative()` to determine if the I64
        /// represents a negative or positive value. Then call `get_magnitude_if_positive()` or
        /// `get_magnitude_if_negative()` to get the magnitude of the number in unsigned u64 format.
        /// This API forces consumers to handle positive and negative numbers safely.
        struct I64 has copy, drop, store {
            negative: bool,
            magnitude: u64,
        }
    }
});
