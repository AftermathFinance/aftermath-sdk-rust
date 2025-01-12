#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Move types for Aftermath's `PythWrapper` package that extends `AfOracle`

use af_sui_pkg_sdk::sui_pkg_sdk;
use sui_framework_sdk::object::UID;

#[cfg(feature = "graphql")]
pub mod graphql;
#[cfg(feature = "ptb")]
pub mod update;

sui_pkg_sdk!(pyth_wrapper {
    module wrapper {
        // =========================================================================
        //  Wrapper Object
        // =========================================================================

        // Shared object representing the wrapper package
        struct PythWrapper has key, store {
            id: UID,
        }

        /// Key type for price feed's source object ID.
        struct PythPriceInfoId has copy, drop, store {}
    }
});
