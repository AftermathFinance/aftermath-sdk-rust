#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Move types for `AftermathFaucet` used in development

use af_sui_pkg_sdk::sui_pkg_sdk;
use sui_framework_sdk::coin::TreasuryCap;
use sui_framework_sdk::object::{ID, UID};

sui_pkg_sdk!(faucet {
    module faucet {
        /* ================= Events ================= */
        struct CreatedFaucet has copy, drop {
            id: ID
        }

        struct AddedCoin<!phantom T> has copy, drop {
            default_mint_amount: u64,
        }

        struct RemovedCoin<!phantom T> has copy, drop {}

        struct SetDefaultMintAmount<!phantom T> has copy, drop {
            default_mint_amount: u64,
        }

        struct MintedCoin<!phantom T> has copy, drop {
            amount: u64,
            user: address
        }

        struct BurnedCoin<!phantom T> has copy, drop {
            amount: u64,
            user: address
        }

        //**************************************************************************************************
        // Faucet
        //**************************************************************************************************

        /// Type that marks the capability to:
        ///   - Remove TreasuryCap's from the Faucet; only if needed if these TreasuryCaps
        ///     need to be moved to a new Faucet address while Devnet remains live .
        struct AdminCap has key { id: UID }

        struct Faucet has key {
            id: UID,
        }

        struct FaucetCoin<!phantom T> has store {
            treasury_cap: TreasuryCap<T>,
            default_mint_amount: u64
        }
    }
});
