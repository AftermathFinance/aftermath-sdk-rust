//! Move types for Aftermath's `SwitchboardWrapper` package that extends `AfOracle`

use af_sui_pkg_sdk::sui_pkg_sdk;
use sui_framework_sdk::UID;

#[cfg(feature = "graphql")]
pub mod graphql;
#[cfg(feature = "ptb")]
pub mod update;

sui_pkg_sdk!(switchboard_wrapper {
    module wrapper {
        // =========================================================================
        //  Wrapper Object
        // =========================================================================

        // Shared object representing the wrapper package
        struct SwitchboardWrapper has key, store {
            id: UID,
        }

        /// Key type for price feed's source object ID.
        struct SwitchboardAggregatorId has copy, drop, store {}
    }
});
