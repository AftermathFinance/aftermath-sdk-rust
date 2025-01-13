// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use sui_sdk_types::CheckpointDigest;

use crate::encoding::Base58;

/// Representation of a network's identifier by the genesis checkpoint's digest
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    derive_more::FromStr,
)]
pub struct ChainIdentifier(CheckpointDigest);

impl ChainIdentifier {
    /// take a short 4 byte identifier and convert it into a ChainIdentifier
    /// short ids come from the JSON RPC getChainIdentifier and are encoded in hex
    pub fn from_chain_short_id(short_id: impl AsRef<str>) -> Option<Self> {
        if hex::encode(Base58::decode(MAINNET_CHAIN_IDENTIFIER_BASE58).ok()?)
            .starts_with(short_id.as_ref())
        {
            Some(get_mainnet_chain_identifier())
        } else if hex::encode(&Base58::decode(TESTNET_CHAIN_IDENTIFIER_BASE58).ok()?)
            .starts_with(short_id.as_ref())
        {
            Some(get_testnet_chain_identifier())
        } else {
            None
        }
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        self.0.inner()
    }

    pub fn mainnet() -> Self {
        get_mainnet_chain_identifier()
    }

    pub fn testnet() -> Self {
        get_testnet_chain_identifier()
    }
}

const MAINNET_CHAIN_IDENTIFIER_BASE58: &str = "4btiuiMPvEENsttpZC7CZ53DruC3MAgfznDbASZ7DR6S";
const TESTNET_CHAIN_IDENTIFIER_BASE58: &str = "69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD";
static MAINNET_CHAIN_IDENTIFIER: OnceLock<ChainIdentifier> = OnceLock::new();
static TESTNET_CHAIN_IDENTIFIER: OnceLock<ChainIdentifier> = OnceLock::new();

fn get_mainnet_chain_identifier() -> ChainIdentifier {
    let digest = MAINNET_CHAIN_IDENTIFIER.get_or_init(|| {
        let digest = CheckpointDigest::new(
            Base58::decode(MAINNET_CHAIN_IDENTIFIER_BASE58)
                .expect("mainnet genesis checkpoint digest literal is invalid")
                .try_into()
                .expect("Mainnet genesis checkpoint digest literal has incorrect length"),
        );
        ChainIdentifier::from(digest)
    });
    *digest
}

fn get_testnet_chain_identifier() -> ChainIdentifier {
    let digest = TESTNET_CHAIN_IDENTIFIER.get_or_init(|| {
        let digest = CheckpointDigest::new(
            Base58::decode(TESTNET_CHAIN_IDENTIFIER_BASE58)
                .expect("testnet genesis checkpoint digest literal is invalid")
                .try_into()
                .expect("Testnet genesis checkpoint digest literal has incorrect length"),
        );
        ChainIdentifier::from(digest)
    });
    *digest
}

impl fmt::Display for ChainIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.inner()[0..4].iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl From<CheckpointDigest> for ChainIdentifier {
    fn from(digest: CheckpointDigest) -> Self {
        Self(digest)
    }
}

impl From<ChainIdentifier> for CheckpointDigest {
    fn from(value: ChainIdentifier) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_identifier_from_short() {
        assert_eq!(
            ChainIdentifier::from_chain_short_id("35834a8a"),
            Some(get_mainnet_chain_identifier())
        );
        assert_eq!(
            ChainIdentifier::from_chain_short_id("4c78adac"),
            Some(get_testnet_chain_identifier())
        );
    }
}
