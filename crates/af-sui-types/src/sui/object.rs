use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};
use sui_sdk_types::Version;

use super::move_object_type::MoveObjectType;
use super::move_package::MovePackage;
use crate::{Address, ObjectId, TransactionDigest};

// =============================================================================
//  Object
// =============================================================================

/// Alias of `sui_types::object::ObjectInner`, skipping the `Arc` in `sui_types::object::Object`.
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Object {
    /// The meat of the object
    data: Data,
    /// The owner that unlocks this object
    pub owner: Owner,
    /// The digest of the transaction that created or last mutated this object
    pub previous_transaction: TransactionDigest,
    /// The amount of SUI we would rebate if this object gets deleted.
    /// This number is re-calculated each time the object is mutated based on
    /// the present storage gas price.
    pub storage_rebate: u64,
}

impl Object {
    /// Its unique on-chain identifier.
    pub fn id(&self) -> ObjectId {
        match &self.data {
            Data::Move(o) => o.id(),
            Data::Package(p) => p.id,
        }
    }

    /// Its current version.
    pub const fn version(&self) -> Version {
        match &self.data {
            Data::Move(o) => o.version,
            Data::Package(p) => p.version,
        }
    }

    /// Reference to the underlying Move object, if it is one.
    pub const fn as_move(&self) -> Option<&MoveObject> {
        let Data::Move(ref obj) = self.data else {
            return None;
        };
        Some(obj)
    }

    /// Reference to the underlying Move package, if it is one.
    pub const fn as_package(&self) -> Option<&MovePackage> {
        let Data::Package(ref obj) = self.data else {
            return None;
        };
        Some(obj)
    }

    /// Convert to the underlying Move object, if it is one.
    pub fn into_move(self) -> Option<MoveObject> {
        let Data::Move(obj) = self.data else {
            return None;
        };
        Some(obj)
    }

    /// Convert to the underlying Move package, if it is one.
    pub fn into_package(self) -> Option<MovePackage> {
        let Data::Package(obj) = self.data else {
            return None;
        };
        Some(obj)
    }
}

// =============================================================================
//  Data
// =============================================================================

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub(crate) enum Data {
    /// An object whose governing logic lives in a published Move module
    Move(MoveObject),
    /// Map from each module name to raw serialized Move module bytes
    Package(MovePackage),
    // ... Sui "native" types go here
}

// =============================================================================
//  Owner
// =============================================================================

/// The entity that owns an object.
// TODO: align this with `sui-sdk-types`; see inner TODOs
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash, Ord, PartialOrd)]
pub enum Owner {
    /// Object is exclusively owned by a single address, and is mutable.
    AddressOwner(Address),
    /// Object is exclusively owned by a single object, and is mutable.
    /// The object ID is converted to Address as SuiAddress is universal.
    // TODO: migrate this to contain an `ObjectId` instead
    ObjectOwner(Address),
    /// Object is shared, can be used by any address, and is mutable.
    // TODO: make this a newtype variant
    Shared {
        /// The version at which the object became shared
        initial_shared_version: Version,
    },
    /// Object is immutable, and hence ownership doesn't matter.
    Immutable,
    /// Object is sequenced via consensus. Ownership is managed by the configured authenticator.
    ///
    /// Note: wondering what happened to `V1`? `Shared` above was the V1 of consensus objects.
    ConsensusV2 {
        /// The version at which the object most recently became a consensus object.
        /// This serves the same function as `initial_shared_version`, except it may change
        /// if the object's Owner type changes.
        start_version: Version,
        /// The authentication mode of the object
        authenticator: Box<Authenticator>,
    },
}

impl Owner {
    /// Only return address of [`AddressOwner`], otherwise return error.
    ///
    /// [`ObjectOwner`]'s address is converted from object id, thus we will skip it.
    ///
    /// [`AddressOwner`]: Owner::AddressOwner
    /// [`ObjectOwner`]: Owner::ObjectOwner
    // TODO: make this return an Option
    pub const fn get_address_owner_address(&self) -> Result<Address, UnexpectedOwnerTypeError> {
        match self {
            Self::AddressOwner(address) => Ok(*address),
            Self::Shared { .. }
            | Self::Immutable
            | Self::ObjectOwner(_)
            | Self::ConsensusV2 { .. } => Err(UnexpectedOwnerTypeError),
        }
    }

    /// This function will return address of both [`AddressOwner`] and [`ObjectOwner`],
    ///
    /// Address of [`ObjectOwner`] is converted from object id, even though the type is [`Address`].
    ///
    /// [`AddressOwner`]: Owner::AddressOwner
    /// [`ObjectOwner`]: Owner::ObjectOwner
    // TODO: make this return an Option
    pub const fn get_owner_address(&self) -> Result<Address, UnexpectedOwnerTypeError> {
        match self {
            Self::AddressOwner(address) | Self::ObjectOwner(address) => Ok(*address),
            Self::Shared { .. } | Self::Immutable | Self::ConsensusV2 { .. } => {
                Err(UnexpectedOwnerTypeError)
            }
        }
    }

    pub const fn is_immutable(&self) -> bool {
        matches!(self, Self::Immutable)
    }

    pub const fn is_address_owned(&self) -> bool {
        matches!(self, Self::AddressOwner(_))
    }

    pub const fn is_child_object(&self) -> bool {
        matches!(self, Self::ObjectOwner(_))
    }

    pub const fn is_shared(&self) -> bool {
        matches!(self, Self::Shared { .. })
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Expecting a single owner, shared ownership found")]
pub struct UnexpectedOwnerTypeError;

impl PartialEq<ObjectId> for Owner {
    fn eq(&self, other: &ObjectId) -> bool {
        match self {
            Self::ObjectOwner(id) => id.inner() == other.inner(),
            Self::AddressOwner(_)
            | Self::Shared { .. }
            | Self::Immutable
            | Self::ConsensusV2 { .. } => false,
        }
    }
}

impl std::fmt::Display for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddressOwner(address) => {
                write!(f, "Account Address ( {} )", address)
            }
            Self::ObjectOwner(address) => {
                write!(f, "Object ID: ( {} )", address)
            }
            Self::Immutable => {
                write!(f, "Immutable")
            }
            Self::Shared { .. } => {
                write!(f, "Shared")
            }
            Self::ConsensusV2 {
                start_version,
                authenticator,
            } => {
                write!(f, "ConsensusV2( {}, {} )", start_version, authenticator)
            }
        }
    }
}

// =============================================================================
//  Authenticator
// =============================================================================

/// An object authenticator for Sui's concensus v2.
///
/// Currently `non_exhaustive` as there is a clear expectation that this will be expanded in the
/// future.
#[derive(Eq, PartialEq, Debug, Clone, Copy, Deserialize, Serialize, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum Authenticator {
    /// The contained Address exclusively has all permissions: read, write, delete, transfer
    SingleOwner(Address),
}

impl Authenticator {
    #[deprecated = "Should be removed soon as this type will evolve and we probably won't be able \
        to guarantee a 1:1 correspondence to an Address"]
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Copied from Mysten's crate + this type may be more complex in the future"
    )]
    pub fn as_single_owner(&self) -> &Address {
        // NOTE: Existing callers are written assuming that only singly-owned
        // ConsensusV2 objects exist. If additional Authenticator variants are
        // added, do not simply panic here. Instead, change the return type of
        // this function and update callers accordingly.
        match self {
            Self::SingleOwner(address) => address,
        }
    }
}

impl std::fmt::Display for Authenticator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingleOwner(address) => {
                write!(f, "SingleOwner({})", address)
            }
        }
    }
}

// =============================================================================
//  MoveObject
// =============================================================================

/// Index marking the end of the object's ID + the beginning of its version
const ID_END_INDEX: usize = ObjectId::LENGTH;

#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MoveObject {
    /// The type of this object. Immutable
    pub type_: MoveObjectType,
    /// DEPRECATED this field is no longer used to determine whether a tx can transfer this
    /// object. Instead, it is always calculated from the objects type when loaded in execution
    pub(crate) has_public_transfer: bool,
    /// Number that increases each time a tx takes this object as a mutable input
    /// This is a lamport timestamp, not a sequentially increasing version
    pub version: Version,
    /// BCS bytes of a Move struct value
    #[serde_as(as = "Bytes")]
    pub contents: Vec<u8>,
}

impl MoveObject {
    /// Get the object's ID from its BCS serialization.
    ///
    /// # Panics
    ///
    /// This will panic if the BCS contents do not enconde a [`MoveObject`].
    pub fn id(&self) -> ObjectId {
        ObjectId::new(
            self.contents[0..ID_END_INDEX]
                .try_into()
                .expect("Corrupted Object BCS"),
        )
    }
}
