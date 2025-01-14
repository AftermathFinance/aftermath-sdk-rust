// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Borrow;
use std::fmt;

use serde::{Deserialize, Serialize};
use sui_sdk_types::types::{Address, StructTag, TypeTag};

use super::gas_coin::{is_gas_coin, Gas};
use crate::{
    IdentStr,
    COIN_METADATA_STRUCT_NAME,
    COIN_MODULE_NAME,
    COIN_STRUCT_NAME,
    COIN_TREASURE_CAP_NAME,
    DYNAMIC_FIELD_FIELD_STRUCT_NAME,
    DYNAMIC_FIELD_MODULE_NAME,
    STAKED_SUI_STRUCT_NAME,
    STAKING_POOL_MODULE_NAME,
    SUI_FRAMEWORK_ADDRESS,
    SUI_SYSTEM_ADDRESS,
};

/// Wrapper around [`StructTag`] with a space-efficient representation for common types like coins.
///
/// The `StructTag` for a gas coin is 84 bytes, so using 1 byte instead is a win.
#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MoveObjectType(MoveObjectType_);

impl fmt::Display for MoveObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let s: StructTag = self.clone().into();
        write!(f, "{s}")
    }
}

impl MoveObjectType {
    pub const fn address(&self) -> Address {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) => SUI_FRAMEWORK_ADDRESS,
            MoveObjectType_::StakedSui => SUI_SYSTEM_ADDRESS,
            MoveObjectType_::Other(s) => s.address,
        }
    }

    pub fn module(&self) -> &IdentStr {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) => COIN_MODULE_NAME,
            MoveObjectType_::StakedSui => STAKING_POOL_MODULE_NAME,
            MoveObjectType_::Other(s) => s.module.borrow(),
        }
    }

    pub fn name(&self) -> &IdentStr {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) => COIN_STRUCT_NAME,
            MoveObjectType_::StakedSui => STAKED_SUI_STRUCT_NAME,
            MoveObjectType_::Other(s) => s.name.borrow(),
        }
    }

    pub fn type_params(&self) -> Vec<TypeTag> {
        match &self.0 {
            MoveObjectType_::GasCoin => vec![Gas::type_tag()],
            MoveObjectType_::StakedSui => vec![],
            MoveObjectType_::Coin(inner) => vec![inner.clone()],
            MoveObjectType_::Other(s) => s.type_params.clone(),
        }
    }

    pub fn into_type_params(self) -> Vec<TypeTag> {
        match self.0 {
            MoveObjectType_::GasCoin => vec![Gas::type_tag()],
            MoveObjectType_::StakedSui => vec![],
            MoveObjectType_::Coin(inner) => vec![inner],
            MoveObjectType_::Other(s) => s.type_params,
        }
    }

    pub fn coin_type_maybe(&self) -> Option<TypeTag> {
        match &self.0 {
            MoveObjectType_::GasCoin => Some(Gas::type_tag()),
            MoveObjectType_::Coin(inner) => Some(inner.clone()),
            MoveObjectType_::StakedSui => None,
            MoveObjectType_::Other(_) => None,
        }
    }

    /// Return true if `self` is `0x2::coin::Coin<T>` for some T (note: T can be SUI)
    pub const fn is_coin(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) => true,
            MoveObjectType_::StakedSui | MoveObjectType_::Other(_) => false,
        }
    }

    /// Return true if `self` is 0x2::coin::Coin<0x2::sui::SUI>
    pub const fn is_gas_coin(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin => true,
            MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) | MoveObjectType_::Other(_) => {
                false
            }
        }
    }

    /// Return true if `self` is `0x2::coin::Coin<t>`
    pub fn is_coin_t(&self, t: &TypeTag) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin => Gas::is_gas_type(t),
            MoveObjectType_::Coin(c) => t == c,
            MoveObjectType_::StakedSui | MoveObjectType_::Other(_) => false,
        }
    }

    pub const fn is_staked_sui(&self) -> bool {
        match &self.0 {
            MoveObjectType_::StakedSui => true,
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) | MoveObjectType_::Other(_) => {
                false
            }
        }
    }

    pub fn is_coin_metadata(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) => {
                false
            }
            MoveObjectType_::Other(s) => {
                s.address == SUI_FRAMEWORK_ADDRESS
                    && Borrow::<IdentStr>::borrow(&s.module) == COIN_MODULE_NAME
                    && Borrow::<IdentStr>::borrow(&s.name) == COIN_METADATA_STRUCT_NAME
            }
        }
    }

    pub fn is_treasury_cap(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) => {
                false
            }
            MoveObjectType_::Other(s) => {
                s.address == SUI_FRAMEWORK_ADDRESS
                    && Borrow::<IdentStr>::borrow(&s.module) == COIN_MODULE_NAME
                    && Borrow::<IdentStr>::borrow(&s.name) == COIN_TREASURE_CAP_NAME
            }
        }
    }

    pub fn is_upgrade_cap(&self) -> bool {
        self.address() == SUI_FRAMEWORK_ADDRESS
            && self.module().as_str() == "package"
            && self.name().as_str() == "UpgradeCap"
    }

    pub fn is_regulated_coin_metadata(&self) -> bool {
        self.address() == SUI_FRAMEWORK_ADDRESS
            && self.module().as_str() == "coin"
            && self.name().as_str() == "RegulatedCoinMetadata"
    }

    pub fn is_coin_deny_cap(&self) -> bool {
        self.address() == SUI_FRAMEWORK_ADDRESS
            && self.module().as_str() == "coin"
            && self.name().as_str() == "DenyCap"
    }

    pub fn is_dynamic_field(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) => {
                false
            }
            MoveObjectType_::Other(s) => {
                s.address == SUI_FRAMEWORK_ADDRESS
                    && Borrow::<IdentStr>::borrow(&s.module) == DYNAMIC_FIELD_MODULE_NAME
                    && Borrow::<IdentStr>::borrow(&s.name) == DYNAMIC_FIELD_FIELD_STRUCT_NAME
            }
        }
    }
}

impl From<StructTag> for MoveObjectType {
    fn from(mut s: StructTag) -> Self {
        Self(if is_gas_coin(&s) {
            MoveObjectType_::GasCoin
        } else if s.is_coin().is_some() {
            // unwrap safe because a coin has exactly one type parameter
            MoveObjectType_::Coin(
                s.type_params
                    .pop()
                    .expect("Coin should have exactly one type parameter"),
            )
        } else if s == StructTag::staked_sui() {
            MoveObjectType_::StakedSui
        } else {
            MoveObjectType_::Other(s)
        })
    }
}

impl From<MoveObjectType> for StructTag {
    fn from(t: MoveObjectType) -> Self {
        match t.0 {
            MoveObjectType_::GasCoin => Self::gas_coin(),
            MoveObjectType_::StakedSui => Self::staked_sui(),
            MoveObjectType_::Coin(inner) => Self::coin(inner),
            MoveObjectType_::Other(s) => s,
        }
    }
}

impl From<MoveObjectType> for TypeTag {
    fn from(o: MoveObjectType) -> Self {
        let s: StructTag = o.into();
        Self::Struct(Box::new(s))
    }
}

/// The internal representation for [`MoveObjectType`].
///
/// It's private to prevent incorrectly constructing an `Other` instead of one of the specialized
/// variants, e.g. `Other(GasCoin::type_())` instead of `GasCoin`
#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Deserialize, Serialize, Hash)]
enum MoveObjectType_ {
    /// A type that is not `0x2::coin::Coin<T>`
    Other(StructTag),
    /// A SUI coin (i.e., `0x2::coin::Coin<0x2::sui::SUI>`)
    GasCoin,
    /// A record of a staked SUI coin (i.e., `0x3::staking_pool::StakedSui`)
    StakedSui,
    /// A non-SUI coin type (i.e., `0x2::coin::Coin<T> where T != 0x2::sui::SUI`)
    Coin(TypeTag),
    // NOTE: if adding a new type here, and there are existing on-chain objects of that
    // type with Other(_), that is ok, but you must hand-roll PartialEq/Eq/Ord/maybe Hash
    // to make sure the new type and Other(_) are interpreted consistently.
}

// =============================================================================
//  Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CLOCK_ID;

    #[test]
    fn clock_object_id_from_str() {
        assert_eq!(Ok(CLOCK_ID), "0x6".parse())
    }

    #[test]
    fn object_id_display_pads() {
        assert_eq!(
            "0x0000000000000000000000000000000000000000000000000000000000000006",
            format!("{}", CLOCK_ID)
        );
    }

    #[test]
    fn sui_address_display_pads() {
        assert_eq!(
            "0x0000000000000000000000000000000000000000000000000000000000000006",
            format!("{}", Address::from(CLOCK_ID))
        );
    }

    #[test]
    fn version_number_display_hex() {
        let version = 0xff_u64;
        assert_eq!("0xff", format!("{version:#x}"))
    }
}
