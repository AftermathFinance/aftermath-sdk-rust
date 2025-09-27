// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
//! Transaction payload pre-signing.
//!
//! A lot of the types here are for compatibility with older APIs.

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use sui_sdk_types::{
    GasPayment,
    Input,
    ObjectReference,
    Transaction,
    TransactionExpiration,
    TransactionKind,
    Version,
};

use crate::encoding::decode_base64_default;
use crate::{Address, ObjectId, ObjectRef};

// =================================================================================================
//  TransactionData
// =================================================================================================

/// The payload that gets sent to the full node as base64 BCS bytes.
#[enum_dispatch(TransactionDataAPI)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    V1(TransactionDataV1),
}

impl TransactionData {
    /// Create a new transaction (V1).
    pub const fn v1(
        kind: TransactionKind,
        sender: Address,
        gas_data: GasData,
        expiration: TransactionExpiration,
    ) -> Self {
        Self::V1(TransactionDataV1 {
            kind,
            sender,
            gas_data,
            expiration,
        })
    }

    /// Get the underlying variant.
    ///
    /// Since the enum is **not** `#[non_exhaustive]` today, this returns a reference. If and when
    /// a new version is introduced, there will have to be a SemVer-breaking update anyway and this
    /// method will return an option.
    pub const fn as_v1(&self) -> &TransactionDataV1 {
        let Self::V1(data) = self;
        data
    }

    /// The payload that gets sent to the full node.
    pub fn encode_base64(&self) -> String {
        crate::encoding::encode_base64_default(
            bcs::to_bytes(self).expect("TransactionData is BCS-compatible"),
        )
    }

    /// Deserialize a transaction from base64 bytes.
    pub fn decode_base64(value: impl AsRef<[u8]>) -> Result<Self, TransactionFromBase64Error> {
        Ok(bcs::from_bytes(&decode_base64_default(value).map_err(
            |e| TransactionFromBase64Error::Base64(e.to_string()),
        )?)?)
    }
}

impl From<TransactionData> for Transaction {
    fn from(
        TransactionData::V1(TransactionDataV1 {
            kind,
            sender,
            gas_data,
            expiration,
        }): TransactionData,
    ) -> Self {
        Self {
            kind,
            sender,
            gas_payment: gas_data.into(),
            expiration,
        }
    }
}

impl From<Transaction> for TransactionData {
    fn from(
        Transaction {
            kind,
            sender,
            gas_payment,
            expiration,
        }: Transaction,
    ) -> Self {
        Self::v1(kind, sender, gas_payment.into(), expiration)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TransactionFromBase64Error {
    #[error(transparent)]
    Bcs(#[from] bcs::Error),
    #[error("Decoding base64 bytes: {0}")]
    Base64(String),
}

// =================================================================================================
//  TransactionDataV1
// =================================================================================================

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionDataV1 {
    pub kind: TransactionKind,
    pub sender: Address,
    pub gas_data: GasData,
    pub expiration: TransactionExpiration,
}

// =================================================================================================
//  GasData
// =================================================================================================

/// Gas payment information for a transaction.
///
/// This type is here for backwards compatibility purposes. The new [`GasPayment`] uses
/// [`ObjectReference`], which is incompatible with [`ObjectRef`] used across our internal Rust
/// APIs (but still compatible at the serde level).
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct GasData {
    pub payment: Vec<ObjectRef>,
    pub owner: Address,
    pub price: u64,
    pub budget: u64,
}

impl From<GasData> for GasPayment {
    fn from(
        GasData {
            payment,
            owner,
            price,
            budget,
        }: GasData,
    ) -> Self {
        Self {
            objects: payment
                .into_iter()
                .map(|(i, v, d)| ObjectReference::new(i, v, d))
                .collect(),
            owner,
            price,
            budget,
        }
    }
}

impl From<GasPayment> for GasData {
    fn from(
        GasPayment {
            objects,
            owner,
            price,
            budget,
        }: GasPayment,
    ) -> Self {
        Self {
            payment: objects.into_iter().map(|oref| oref.into_parts()).collect(),
            owner,
            price,
            budget,
        }
    }
}

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
        id: ObjectId,
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
            } => Self::Shared {
                object_id: id,
                initial_shared_version,
                mutable,
            },
            ObjectArg::Receiving((i, v, d)) => Self::Receiving(ObjectReference::new(i, v, d)),
        }
    }
}

impl ObjectArg {
    /// Argument for transactions acquiring an immutable reference to the network clock.
    ///
    /// Only system transactions acquire mutable references to the clock.
    pub const CLOCK_IMM: Self = Self::SharedObject {
        id: crate::object_id(b"0x6"),
        initial_shared_version: 1,
        mutable: false,
    };

    /// Argument for transactions acquiring an immutable reference to the system state.
    pub const SYSTEM_STATE_IMM: Self = Self::SharedObject {
        id: crate::object_id(b"0x5"),
        initial_shared_version: 1,
        mutable: false,
    };

    /// Argument for transactions acquiring a mutable reference to the system state.
    pub const SYSTEM_STATE_MUT: Self = Self::SharedObject {
        id: crate::object_id(b"0x5"),
        initial_shared_version: 1,
        mutable: true,
    };

    pub const fn id(&self) -> ObjectId {
        match self {
            Self::ImmOrOwnedObject((id, ..)) => *id,
            Self::SharedObject { id, .. } => *id,
            Self::Receiving((id, ..)) => *id,
        }
    }

    pub const fn id_borrowed(&self) -> &ObjectId {
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

// =================================================================================================
//  Traits
// =================================================================================================

#[enum_dispatch]
pub trait TransactionDataAPI {
    fn sender(&self) -> Address;

    fn kind(&self) -> &TransactionKind;

    fn kind_mut(&mut self) -> &mut TransactionKind;

    fn into_kind(self) -> TransactionKind;

    fn gas_data(&self) -> &GasData;

    fn gas_owner(&self) -> Address;

    fn gas(&self) -> &[ObjectRef];

    fn gas_price(&self) -> u64;

    fn gas_budget(&self) -> u64;

    fn expiration(&self) -> &TransactionExpiration;

    fn is_system_tx(&self) -> bool;

    fn is_genesis_tx(&self) -> bool;

    /// returns true if the transaction is one that is specially sequenced to run at the very end
    /// of the epoch
    fn is_end_of_epoch_tx(&self) -> bool;

    /// Check if the transaction is sponsored (namely gas owner != sender)
    fn is_sponsored_tx(&self) -> bool;

    fn gas_data_mut(&mut self) -> &mut GasData;
}

impl TransactionDataAPI for TransactionDataV1 {
    fn sender(&self) -> Address {
        self.sender
    }

    fn kind(&self) -> &TransactionKind {
        &self.kind
    }

    fn kind_mut(&mut self) -> &mut TransactionKind {
        &mut self.kind
    }

    fn into_kind(self) -> TransactionKind {
        self.kind
    }

    fn gas_data(&self) -> &GasData {
        &self.gas_data
    }

    fn gas_owner(&self) -> Address {
        self.gas_data.owner
    }

    fn gas(&self) -> &[ObjectRef] {
        &self.gas_data.payment
    }

    fn gas_price(&self) -> u64 {
        self.gas_data.price
    }

    fn gas_budget(&self) -> u64 {
        self.gas_data.budget
    }

    fn expiration(&self) -> &TransactionExpiration {
        &self.expiration
    }

    /// Check if the transaction is sponsored (namely gas owner != sender)
    fn is_sponsored_tx(&self) -> bool {
        self.gas_owner() != self.sender
    }

    fn is_end_of_epoch_tx(&self) -> bool {
        matches!(
            self.kind,
            TransactionKind::ChangeEpoch(_) | TransactionKind::EndOfEpoch(_)
        )
    }

    fn is_system_tx(&self) -> bool {
        // Keep this as an exhaustive match so that we can't forget to update it.
        match self.kind {
            TransactionKind::ChangeEpoch(_)
            | TransactionKind::Genesis(_)
            | TransactionKind::ConsensusCommitPrologue(_)
            | TransactionKind::ConsensusCommitPrologueV2(_)
            | TransactionKind::ConsensusCommitPrologueV3(_)
            | TransactionKind::ConsensusCommitPrologueV4(_)
            | TransactionKind::AuthenticatorStateUpdate(_)
            | TransactionKind::RandomnessStateUpdate(_)
            | TransactionKind::ProgrammableSystemTransaction(_)
            | TransactionKind::EndOfEpoch(_) => true,
            TransactionKind::ProgrammableTransaction(_) => false,
        }
    }

    fn is_genesis_tx(&self) -> bool {
        matches!(self.kind, TransactionKind::Genesis(_))
    }

    fn gas_data_mut(&mut self) -> &mut GasData {
        &mut self.gas_data
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn transaction_data_base64_roundtrip(transaction: sui_sdk_types::Transaction) {
        let original: TransactionData = transaction.into();
        let reconstructed = TransactionData::decode_base64(original.encode_base64())
            .expect("Valid base64 transaction");
        assert_eq!(original, reconstructed);
    }
}
