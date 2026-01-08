// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
//! Transaction payload pre-signing.
//!
//! A lot of the types here are for compatibility with older APIs.

use serde::{Deserialize, Serialize};
use sui_sdk_types::{Input, Mutability, ObjectReference, SharedInput, Version};

use crate::{Address, ObjectRef};

// =================================================================================================
//  ObjectArg
// =================================================================================================

/// Object argument for a programmable transaction.
///
/// This type is here for backwards compatibility purposes; specifically to use in our programmable
/// transaction builder. The actual [`ProgrammableTransaction`] does not contain this type.
///
/// [`ProgrammableTransaction`]: crate::ProgrammableTransaction
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ObjectArg {
    /// A Move object from fastpath.
    ImmOrOwnedObject(ObjectRef),
    /// A Move object from consensus (historically consensus objects were always shared).
    ///
    /// SharedObject::mutable controls whether caller asks for a mutable reference to shared object.
    SharedObject {
        id: Address,
        initial_shared_version: Version,
        mutable: bool,
    },
    /// A Move object that can be received in this transaction.
    Receiving(ObjectRef),
}

impl From<ObjectArg> for Input {
    fn from(value: ObjectArg) -> Self {
        match value {
            ObjectArg::ImmOrOwnedObject((i, v, d)) => {
                Self::ImmutableOrOwned(ObjectReference::new(i, v, d))
            }
            ObjectArg::SharedObject {
                id,
                initial_shared_version,
                mutable,
            } => Self::Shared(SharedInput::new(
                id,
                initial_shared_version,
                Mutability::from(mutable),
            )),

            ObjectArg::Receiving((i, v, d)) => Self::Receiving(ObjectReference::new(i, v, d)),
        }
    }
}

impl ObjectArg {
    /// Argument for transactions acquiring an immutable reference to the network clock.
    ///
    /// Only system transactions acquire mutable references to the clock.
    pub const CLOCK_IMM: Self = Self::SharedObject {
        id: Address::from_static("0x6"),
        initial_shared_version: 1,
        mutable: false,
    };

    /// Argument for transactions acquiring an immutable reference to the system state.
    pub const SYSTEM_STATE_IMM: Self = Self::SharedObject {
        id: Address::from_static("0x5"),
        initial_shared_version: 1,
        mutable: false,
    };

    /// Argument for transactions acquiring a mutable reference to the system state.
    pub const SYSTEM_STATE_MUT: Self = Self::SharedObject {
        id: Address::from_static("0x5"),
        initial_shared_version: 1,
        mutable: true,
    };

    pub const fn id(&self) -> Address {
        match self {
            Self::ImmOrOwnedObject((id, ..)) => *id,
            Self::SharedObject { id, .. } => *id,
            Self::Receiving((id, ..)) => *id,
        }
    }

    pub const fn id_borrowed(&self) -> &Address {
        match self {
            Self::ImmOrOwnedObject((id, ..)) => id,
            Self::SharedObject { id, .. } => id,
            Self::Receiving((id, ..)) => id,
        }
    }

    /// For shared object arguments: set their `mutable` flag value.
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Not changing the public API right now"
    )]
    pub fn set_mutable(&mut self, mutable_: bool) -> Result<(), ImmOwnedOrReceivingError> {
        match self {
            Self::SharedObject { mutable, .. } => {
                *mutable = mutable_;
                Ok(())
            }
            _ => Err(ImmOwnedOrReceivingError),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Only Shared ObjectArg's have a mutable flag")]
pub struct ImmOwnedOrReceivingError;
