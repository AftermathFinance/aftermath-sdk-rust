use sui_sdk_types::{Object, Owner, Version};

use crate::Address;

// =============================================================================
//  Object
// =============================================================================

pub trait ObjectHelpers {
    #[cfg(feature = "hash")]
    /// Input for transactions that interact with this object.
    fn object_arg(&self, mutable: bool) -> crate::ObjectArg;
}

impl ObjectHelpers for Object {
    #[cfg(feature = "hash")]
    fn object_arg(&self, mutable: bool) -> crate::ObjectArg {
        use Owner::*;
        let id = self.object_id();
        match self.owner() {
            Address(_) | Object(_) | Immutable => {
                crate::ObjectArg::ImmOrOwnedObject((id, self.version(), self.digest()))
            }
            Shared(initial_shared_version)
            | ConsensusAddress {
                start_version: initial_shared_version,
                ..
            } => crate::ObjectArg::SharedObject {
                id,
                initial_shared_version: *initial_shared_version,
                mutable,
            },
        }
    }
}

pub trait OwnerHelpers {
    /// Only return address of [`AddressOwner`], otherwise return error.
    ///
    /// [`ObjectOwner`]'s address is converted from object id, thus we will skip it.
    ///
    /// [`AddressOwner`]: Owner::AddressOwner
    /// [`ObjectOwner`]: Owner::ObjectOwner
    fn get_address_owner_address(&self) -> Option<Address>;

    /// This function will return address of [`AddressOwner`], [`ObjectOwner`] and [`ConsensusAddress`],
    ///
    /// Address of [`ObjectOwner`] is converted from object id, even though the type is [`Address`].
    ///
    /// [`AddressOwner`]: Owner::AddressOwner
    /// [`ObjectOwner`]: Owner::ObjectOwner
    fn get_owner_address(&self) -> Option<Address>;

    fn is_immutable(&self) -> bool;

    fn is_address_owned(&self) -> bool;

    fn is_child_object(&self) -> bool;

    fn is_shared(&self) -> bool;

    /// Either the `initial_shared_version` for a [`Shared`] object or the `start_version` of a
    /// [`ConsensusV2`] one.
    ///
    /// [`Shared`]: Owner::Shared
    /// [`ConsensusV2`]: Owner::ConsensusV2
    fn start_version(&self) -> Option<Version>;
}

impl OwnerHelpers for Owner {
    fn get_address_owner_address(&self) -> Option<Address> {
        match self {
            Self::Address(address) => Some(*address),
            Self::Shared { .. }
            | Self::Immutable
            | Self::Object(_)
            | Self::ConsensusAddress { .. } => None,
        }
    }

    fn get_owner_address(&self) -> Option<Address> {
        match self {
            Self::Address(address) => Some(*address),
            Self::Object(id) => Some(*id.as_address()),
            Self::ConsensusAddress { owner, .. } => Some(*owner),
            Self::Shared { .. } | Self::Immutable => None,
        }
    }

    fn is_immutable(&self) -> bool {
        matches!(self, Self::Immutable)
    }

    fn is_address_owned(&self) -> bool {
        matches!(self, Self::Address(_))
    }

    fn is_child_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    fn is_shared(&self) -> bool {
        matches!(self, Self::Shared { .. })
    }

    fn start_version(&self) -> Option<Version> {
        match self {
            Self::Immutable | Self::Object(_) | Self::Address(_) => None,
            Self::Shared(version) => Some(*version),
            Self::ConsensusAddress { start_version, .. } => Some(*start_version),
        }
    }
}
