//! Stork price update data, asset identifiers, and BCS serialization helpers.

use crate::{ParseError, U256, keccak::keccak256};
use bytes::{BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use std::array::TryFromSliceError;

pub use crate::stork_small_id::SmallAssetId;

/// An asset id as used by Stork, e.g. "BTCUSD" or "ETHUSD".
///
/// List of available asset ids and their encodings can be found at
/// <https://docs.stork.network/resources/asset-id-registry>
pub type AssetId = String;

/// Keccak256-encoded asset id, as used in Stork price updates.
///
/// For example, "BTCUSD" is encoded as
/// 0x7404e3d104ea7841c3d9e6fd20adfe99b4ad586bc08d8f3bd3afef894cf184de. Displayed, parsed and
/// serialized like its [`U256`].
///
/// List of available asset ids and their encodings can be found at
/// <https://docs.stork.network/resources/asset-id-registry>
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[repr(transparent)]
pub struct EncodedAssetId(pub U256);

impl EncodedAssetId {
    /// Derives the Keccak256-encoded asset id from a regular asset id string.
    pub fn from_asset_id<S: AsRef<str>>(asset_id: S) -> Self {
        Self(U256::from(keccak256(asset_id.as_ref().as_bytes())))
    }
    /// The 32 bytes, e.g. for the on-chain Stork `EncodedAssetId`.
    pub const fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<[u8; 32]> for EncodedAssetId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(U256::from(bytes))
    }
}

impl AsRef<[u8]> for EncodedAssetId {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl TryFrom<&[u8]> for EncodedAssetId {
    type Error = TryFromSliceError;
    /// Takes exactly 32 bytes, e.g. those of the on-chain Stork `EncodedAssetId`.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        U256::try_from(bytes).map(Self)
    }
}

impl std::fmt::Display for EncodedAssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <U256 as std::fmt::Display>::fmt(&self.0, f)
    }
}

impl std::str::FromStr for EncodedAssetId {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<U256>().map(Self)
    }
}

/// A signed Stork price update for one asset, with the fields the on-chain Stork contract needs to
/// verify it.
///
/// This flattens the `stork_signed_price` of a Stork API price message; its serde and BCS forms
/// are this crate's own, not the Stork API's JSON. See
/// <https://docs.stork.network/getting-started/becoming-a-subscriber> for the API.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PriceUpdate {
    /// Keccak256-encoded asset id.
    pub id: EncodedAssetId,
    /// Unix timestamp in nanoseconds.
    pub timestamp_ns: u64,
    /// Price multiplied by 10^18.
    pub quantized_value_magnitude: u128,
    /// Sign of the price.
    pub quantized_value_negative: bool,
    /// Publisher merkle root. Needed for on-chain verification.
    pub publisher_merkle_root: U256,
    /// Checksum for aggregation function. Needed for on-chain verification.
    pub value_compute_alg_hash: U256,
    /// r component of ECDSA secp256k1 signature. Needed for on-chain verification.
    pub r: U256,
    /// s component of ECDSA secp256k1 signature. Needed for on-chain verification.
    pub s: U256,
    /// v component of ECDSA secp256k1 signature. Needed for on-chain verification.
    pub v: u8,
}

impl TryFrom<BytesMut> for PriceUpdate {
    type Error = std::io::Error;
    /// Decodes the BCS representation, like [`PriceUpdate::from_slice`].
    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        Self::from_slice(&bytes)
    }
}

impl PriceUpdate {
    /// Decodes the BCS representation written by [`Self::serialize_into`].
    pub fn from_slice(bytes: &[u8]) -> Result<Self, std::io::Error> {
        bcs::from_bytes(bytes)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
    /// Length of the BCS representation.
    pub fn serialized_size(&self) -> usize {
        bcs::serialized_size(self).expect("BCS-serializable")
    }
    /// Writes the BCS representation, [`Self::serialized_size`] bytes long.
    pub fn serialize_into<W: BufMut>(&self, w: &mut W) {
        let mut writer = w.writer();
        bcs::serialize_into(&mut writer, self).expect("BCS-serializable");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stork_encoded_asset_id() {
        assert_eq!(
            EncodedAssetId::from_asset_id("BTCUSD"),
            EncodedAssetId(U256::strict_from_str(
                "0x7404e3d104ea7841c3d9e6fd20adfe99b4ad586bc08d8f3bd3afef894cf184de"
            )),
        );
        assert_eq!(
            EncodedAssetId::from_asset_id("ETHUSD"),
            EncodedAssetId(U256::strict_from_str(
                "0x59102b37de83bdda9f38ac8254e596f0d9ac61d2035c07936675e87342817160"
            )),
        );
    }

    #[test]
    fn stork_bytes_serde() {
        let asset_id = SmallAssetId::intern("BTCUSD");
        let update = PriceUpdate {
            id: EncodedAssetId::from_asset_id(asset_id),
            timestamp_ns: 1_234_567_890,
            quantized_value_magnitude: 12_345_678_901_234_567_890,
            quantized_value_negative: true,
            publisher_merkle_root: U256::from(42),
            value_compute_alg_hash: U256::from(43),
            r: U256::from(44),
            s: U256::from(45),
            v: 27,
        };
        let mut bytes = BytesMut::with_capacity(update.serialized_size());
        update.serialize_into(&mut bytes);
        assert_eq!(bytes.len(), update.serialized_size());
        assert_eq!(PriceUpdate::from_slice(&bytes).unwrap(), update);
        let deserialized = PriceUpdate::try_from(bytes).unwrap();
        assert_eq!(update, deserialized);
    }

    #[test]
    fn encoded_asset_id_from_bytes() {
        let id = EncodedAssetId::from_asset_id("BTCUSD");

        assert_eq!(EncodedAssetId::try_from(id.as_slice()).unwrap(), id);
        assert_eq!(EncodedAssetId::from(id.0.into_inner()), id);
        assert_eq!(id.as_ref(), id.0.as_slice());
        assert!(EncodedAssetId::try_from(&id.0.as_slice()[1..]).is_err());
        assert!(EncodedAssetId::try_from([0; 33].as_slice()).is_err());
    }

    #[test]
    fn encoded_asset_id_formats_like_u256() {
        let id = EncodedAssetId::from_asset_id("BTCUSD");
        let hex = "0x7404e3d104ea7841c3d9e6fd20adfe99b4ad586bc08d8f3bd3afef894cf184de";

        assert_eq!(id.to_string(), hex);
        assert_eq!(hex.parse(), Ok(id));
        assert_eq!(serde_json::to_string(&id).unwrap(), format!("\"{hex}\""));
        assert_eq!(
            serde_json::from_str::<EncodedAssetId>(&format!("\"{hex}\"")).unwrap(),
            id
        );
        assert_eq!(bcs::to_bytes(&id).unwrap(), bcs::to_bytes(&id.0).unwrap());
    }
}
