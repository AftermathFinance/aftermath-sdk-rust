#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]
// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
//
// Includes a lot of the constants in the original `sui-types`
// https://github.com/MystenLabs/sui/tree/main/crates/sui-types

//! Aftermath's extensions to [`sui_sdk_types`].
//!
//! Includes some types and constants from the original [`sui_types`] and [`move_core_types`] that
//! are not present in `sui_sdk_types`. This crate also re-exports a lot of the types in
//! `sui_sdk_types`.
//!
//! This crate was originally conceived with the following objectives:
//! - [`serde`] compatibility with the full Sui checkpoint data
//! - avoiding dynamic error types so that callers could match against errors and react accordingly
//! - using a minimal set of dependencies
//! - [SemVer](https://doc.rust-lang.org/cargo/reference/semver.html) compatibility
//!
//! <div class="warning">
//!
//! The long-term plan is to deprecate most of this in favor of [`sui_sdk_types`]. However, there
//! are some types in that crate that don't expose all of the attributes/methods we need yet.
//!
//! </div>
//!
//! [`serde`]: https://docs.rs/serde/latest/serde/index.html
//! [`sui_types`]: https://mystenlabs.github.io/sui/sui_types/index.html
//! [`move_core_types`]: https://github.com/MystenLabs/sui/tree/main/external-crates/move/crates/move-core-types
//! [`sui_sdk_types`]: https://docs.rs/sui-sdk-types/latest/sui_sdk_types/

#[doc(no_inline)]
pub use sui_sdk_types::{
    ActiveJwk,
    Address,
    Argument,
    CheckpointCommitment,
    CheckpointContents,
    CheckpointContentsDigest,
    CheckpointDigest,
    CheckpointSequenceNumber,
    CheckpointSummary,
    CheckpointTimestamp,
    Command,
    ConsensusCommitDigest,
    Digest,
    EffectsAuxiliaryDataDigest,
    EndOfEpochData,
    EpochId,
    Event,
    ExecutionError,
    ExecutionStatus,
    GasCostSummary,
    IdOperation,
    Identifier,
    Jwk,
    JwkId,
    MoveCall,
    ObjectDigest,
    ObjectId,
    ObjectIn,
    ObjectOut,
    ProgrammableTransaction,
    ProtocolVersion,
    SignedTransaction,
    StructTag,
    Transaction,
    TransactionDigest,
    TransactionEffects,
    TransactionEffectsDigest,
    TransactionEffectsV1,
    TransactionEffectsV2,
    TransactionEvents,
    TransactionEventsDigest,
    TransactionExpiration,
    TransactionKind,
    TypeTag,
    UnchangedSharedKind,
    UserSignature,
    Version,
};

mod const_address;
pub mod encoding;
#[cfg(feature = "hash")]
mod hash;
/// Aftermath's versions of [`move_core_types`](https://github.com/MystenLabs/sui/tree/main/external-crates/move/crates/move-core-types).
pub(crate) mod move_core;
/// Aftermath's versions of [`sui_types`](https://mystenlabs.github.io/sui/sui_types/index.html).
pub mod sui;

#[doc(inline)]
pub use self::move_core::identifier::{IdentStr, InvalidIdentifierError};
#[cfg(feature = "u256")]
#[doc(inline)]
pub use self::move_core::u256::{self, U256};
#[doc(inline)]
pub use self::sui::chain_identifier::ChainIdentifier;
#[doc(inline)]
pub use self::sui::effects::TransactionEffectsAPI;
#[doc(inline)]
pub use self::sui::full_checkpoint_content::{CheckpointData, CheckpointTransaction};
#[doc(inline)]
pub use self::sui::move_object_type::MoveObjectType;
#[doc(inline)]
pub use self::sui::move_package::{MovePackage, TypeOrigin, UpgradeInfo};
#[doc(inline)]
pub use self::sui::object::{MoveObject, Object, Owner};
#[doc(inline)]
pub use self::sui::transaction::{
    GasData,
    ObjectArg,
    TransactionData,
    TransactionDataAPI,
    TransactionDataV1,
};

// =============================================================================
//  Aliases
// =============================================================================

/// Reference to a particular object.
///
/// Used for immutable or owned object transaction inputs (fast-path).
///
/// Can be created from [`ObjectReference::into_parts`].
///
/// [`ObjectReference::into_parts`]: sui_sdk_types::ObjectReference::into_parts
pub type ObjectRef = (ObjectId, Version, ObjectDigest);

// =============================================================================
//  Constants
// =============================================================================

/// Object ID of the onchain `Clock`.
pub const CLOCK_ID: ObjectId = ObjectId::new(hex_address_bytes(b"0x6"));

const OBJECT_DIGEST_DELETED_BYTE_VAL: u8 = 99;
const OBJECT_DIGEST_WRAPPED_BYTE_VAL: u8 = 88;
const OBJECT_DIGEST_CANCELLED_BYTE_VAL: u8 = 77;

/// A marker that signifies the object is deleted.
pub const OBJECT_DIGEST_DELETED: ObjectDigest =
    ObjectDigest::new([OBJECT_DIGEST_DELETED_BYTE_VAL; 32]);

/// A marker that signifies the object is wrapped into another object.
pub const OBJECT_DIGEST_WRAPPED: ObjectDigest =
    ObjectDigest::new([OBJECT_DIGEST_WRAPPED_BYTE_VAL; 32]);

pub const OBJECT_DIGEST_CANCELLED: ObjectDigest =
    ObjectDigest::new([OBJECT_DIGEST_CANCELLED_BYTE_VAL; 32]);

pub const COIN_MODULE_NAME: &IdentStr = IdentStr::cast("coin");
pub const COIN_STRUCT_NAME: &IdentStr = IdentStr::cast("Coin");
pub const COIN_METADATA_STRUCT_NAME: &IdentStr = IdentStr::cast("CoinMetadata");
pub const COIN_TREASURE_CAP_NAME: &IdentStr = IdentStr::cast("TreasuryCap");

pub const PAY_MODULE_NAME: &IdentStr = IdentStr::cast("pay");
pub const PAY_JOIN_FUNC_NAME: &IdentStr = IdentStr::cast("join");
pub const PAY_SPLIT_N_FUNC_NAME: &IdentStr = IdentStr::cast("divide_and_keep");
pub const PAY_SPLIT_VEC_FUNC_NAME: &IdentStr = IdentStr::cast("split_vec");

pub const DYNAMIC_FIELD_MODULE_NAME: &IdentStr = IdentStr::cast("dynamic_field");
pub const DYNAMIC_FIELD_FIELD_STRUCT_NAME: &IdentStr = IdentStr::cast("Field");

pub const DYNAMIC_OBJECT_FIELD_MODULE_NAME: &IdentStr = IdentStr::cast("dynamic_object_field");
pub const DYNAMIC_OBJECT_FIELD_WRAPPER_STRUCT_NAME: &IdentStr = IdentStr::cast("Wrapper");

pub const MIST_PER_SUI: u64 = 1_000_000_000;

/// Total supply denominated in Sui
pub const TOTAL_SUPPLY_SUI: u64 = 10_000_000_000;

// Note: cannot use checked arithmetic here since `const unwrap` is still unstable.
/// Total supply denominated in Mist
pub const TOTAL_SUPPLY_MIST: u64 = TOTAL_SUPPLY_SUI * MIST_PER_SUI;

pub const GAS_MODULE_NAME: &IdentStr = IdentStr::cast("sui");
pub const GAS_STRUCT_NAME: &IdentStr = IdentStr::cast("SUI");

/// Maximum number of active validators at any moment.
/// We do not allow the number of validators in any epoch to go above this.
pub const MAX_VALIDATOR_COUNT: u64 = 150;

/// Lower-bound on the amount of stake required to become a validator.
///
/// 30 million SUI
pub const MIN_VALIDATOR_JOINING_STAKE_MIST: u64 = 30_000_000 * MIST_PER_SUI;

/// Threshold below which validators enter a grace period to be removed.
///
/// Validators with stake amount below `validator_low_stake_threshold` are considered to
/// have low stake and will be escorted out of the validator set after being below this
/// threshold for more than `validator_low_stake_grace_period` number of epochs.
///
/// 20 million SUI
pub const VALIDATOR_LOW_STAKE_THRESHOLD_MIST: u64 = 20_000_000 * MIST_PER_SUI;

/// Validators with stake below `validator_very_low_stake_threshold` will be removed
/// immediately at epoch change, no grace period.
///
/// 15 million SUI
pub const VALIDATOR_VERY_LOW_STAKE_THRESHOLD_MIST: u64 = 15_000_000 * MIST_PER_SUI;

/// A validator can have stake below `validator_low_stake_threshold`
/// for this many epochs before being kicked out.
pub const VALIDATOR_LOW_STAKE_GRACE_PERIOD: u64 = 7;

pub const STAKING_POOL_MODULE_NAME: &IdentStr = IdentStr::cast("staking_pool");
pub const STAKED_SUI_STRUCT_NAME: &IdentStr = IdentStr::cast("StakedSui");

pub const ADD_STAKE_MUL_COIN_FUN_NAME: &IdentStr = IdentStr::cast("request_add_stake_mul_coin");
pub const ADD_STAKE_FUN_NAME: &IdentStr = IdentStr::cast("request_add_stake");
pub const WITHDRAW_STAKE_FUN_NAME: &IdentStr = IdentStr::cast("request_withdraw_stake");

macro_rules! built_in_ids {
    ($($addr:ident / $id:ident = $init:expr);* $(;)?) => {
        $(
            pub const $addr: Address = builtin_address($init);
            pub const $id: ObjectId = ObjectId::new($addr.into_inner());
        )*
    }
}

macro_rules! built_in_pkgs {
    ($($addr:ident / $id:ident = $init:expr);* $(;)?) => {
        built_in_ids! { $($addr / $id = $init;)* }
        pub const SYSTEM_PACKAGE_ADDRESSES: &[Address] = &[$($addr),*];
        pub fn is_system_package(addr: impl Into<Address>) -> bool {
            matches!(addr.into(), $($addr)|*)
        }
    }
}

built_in_pkgs! {
    MOVE_STDLIB_ADDRESS / MOVE_STDLIB_PACKAGE_ID = 0x1;
    SUI_FRAMEWORK_ADDRESS / SUI_FRAMEWORK_PACKAGE_ID = 0x2;
    SUI_SYSTEM_ADDRESS / SUI_SYSTEM_PACKAGE_ID = 0x3;
    BRIDGE_ADDRESS / BRIDGE_PACKAGE_ID = 0xb;
    DEEPBOOK_ADDRESS / DEEPBOOK_PACKAGE_ID = 0xdee9;
}

const fn builtin_address(suffix: u16) -> Address {
    let mut addr = [0u8; Address::LENGTH];
    let [hi, lo] = suffix.to_be_bytes();
    addr[Address::LENGTH - 2] = hi;
    addr[Address::LENGTH - 1] = lo;
    Address::new(addr)
}

// =============================================================================
//  Functions
// =============================================================================

#[doc(inline)]
pub use self::const_address::hex_address_bytes;
#[doc(inline)]
pub use self::encoding::{decode_base64_default, encode_base64_default};

/// `const`-ructor for Sui addresses.
pub const fn address(bytes: &[u8]) -> Address {
    Address::new(hex_address_bytes(bytes))
}

/// `const`-ructor for object IDs.
pub const fn object_id(bytes: &[u8]) -> ObjectId {
    ObjectId::new(hex_address_bytes(bytes))
}
