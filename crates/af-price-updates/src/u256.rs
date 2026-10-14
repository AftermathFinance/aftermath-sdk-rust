use crate::ParseError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

const LENGTH: usize = 32;
type HexBuffer<const PREFIX: bool> = const_hex::Buffer<LENGTH, PREFIX>;

/// A 256-bit value stored as 32 big-endian bytes, such as a Pyth price feed ID or a Stork encoded
/// asset ID.
///
/// Displayed as `0x`-prefixed hex and parsed from up to 64 hex digits with an optional `0x` prefix.
/// Serialized as such a hex string in human-readable formats, which deserialize like
/// [`str::parse`], and as 32 bytes otherwise.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U256([u8; LENGTH]);

impl U256 {
    /// Zero.
    pub const ZERO: Self = Self([0; LENGTH]);

    /// Constructs from a hex string, optionally `0x`-prefixed, in const contexts.
    ///
    /// # Panics
    ///
    /// If `hex` is empty, longer than 64 digits or not hex. Outside const contexts, prefer
    /// [`str::parse`], which returns an error instead.
    pub const fn strict_from_str(hex: &str) -> Self {
        let hex = remove_0x_prefix(hex.as_bytes());
        if hex.is_empty() {
            panic!("input hex string must be non-empty");
        } else if hex.len() > 2 * LENGTH {
            panic!("input hex string is too long for U256");
        }
        let mut buf = [b'0'; 2 * LENGTH];
        let (_, part) = buf.split_at_mut(2 * LENGTH - hex.len());
        let mut i = 0;
        while i < hex.len() {
            part[i] = hex[i];
            i += 1;
        }
        match const_hex::const_decode_to_array(&buf) {
            Err(_) => panic!("input hex string has wrong character"),
            Ok(addr) => Self(addr),
        }
    }
    /// The big-endian bytes.
    pub const fn into_inner(self) -> [u8; LENGTH] {
        self.0
    }
    /// The big-endian bytes.
    pub const fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl From<u64> for U256 {
    /// Constructs an U256 from a `u64` `value` mainly for testing purposes.
    fn from(value: u64) -> Self {
        let mut bytes = [0u8; LENGTH];
        bytes[LENGTH - 8..].copy_from_slice(&value.to_be_bytes());
        Self(bytes)
    }
}

impl From<[u8; LENGTH]> for U256 {
    fn from(bytes: [u8; LENGTH]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for U256 {
    type Error = std::array::TryFromSliceError;
    /// Takes exactly 32 big-endian bytes.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        <[u8; LENGTH]>::try_from(bytes).map(Self)
    }
}

impl From<U256> for [u8; LENGTH] {
    fn from(value: U256) -> Self {
        value.0
    }
}

impl AsRef<[u8]> for U256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut buffer = HexBuffer::<true>::new();
        let hex = buffer.format(&self.0);
        f.write_str(hex)
    }
}

impl std::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut buffer = HexBuffer::<true>::new();
        let hex = buffer.format(&self.0);
        f.write_str(hex)
    }
}

impl FromStr for U256 {
    type Err = ParseError;
    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        let hex = remove_0x_prefix(hex.as_bytes());
        if hex.is_empty() || hex.len() > 2 * LENGTH {
            return Err(ParseError::new("expected 1 to 64 hex digits"));
        }
        let mut buf = [b'0'; 2 * LENGTH];
        buf[2 * LENGTH - hex.len()..].copy_from_slice(hex);
        const_hex::decode_to_array(buf)
            .map(Self)
            .map_err(|err| match err {
                // The index is into the zero-padded digits, not the input, so it is left out.
                const_hex::FromHexError::InvalidHexCharacter { c, .. } => {
                    ParseError::new(format!("invalid hex character {c:?}"))
                }
                err => ParseError::new(format!("invalid hex: {err}")),
            })
    }
}

impl<'de> Deserialize<'de> for U256 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            struct HexVisitor;

            impl serde::de::Visitor<'_> for HexVisitor {
                type Value = U256;
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("up to 64 hex digits with an optional 0x prefix")
                }
                fn visit_str<E: serde::de::Error>(self, hex: &str) -> Result<Self::Value, E> {
                    hex.parse().map_err(E::custom)
                }
            }

            deserializer.deserialize_str(HexVisitor)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(Deserialize)]
            #[serde(rename = "U256")]
            struct Value([u8; LENGTH]);

            let value = Value::deserialize(deserializer)?;
            Ok(Self(value.0))
        }
    }
}

impl Serialize for U256 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut buffer = HexBuffer::<true>::new();
            let hex = buffer.format(&self.0);
            hex.serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("U256", &self.0)
        }
    }
}

/// Removes initial "0x" prefix if any.
#[inline]
const fn remove_0x_prefix(hex: &[u8]) -> &[u8] {
    match hex {
        [b'0', b'x', rest @ ..] => rest,
        _ => hex,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u256_strict_from_str() {
        assert_eq!(U256::from(0x0), U256::strict_from_str("0x0"));
        assert_eq!(U256::from(0x1), U256::strict_from_str("0x1"));
        assert_eq!(U256::from(0x10), U256::strict_from_str("0x10"));
        assert_eq!(U256::from(0x999), U256::strict_from_str("0x999"));
        assert_eq!(U256::from(0xabcd), U256::strict_from_str("0xabcd"));
        assert_eq!(U256::from(0xab52_cdef), U256::strict_from_str("0xab52cdef"));
    }

    #[test]
    fn human_readable_serde_parses_like_from_str() {
        let full = "0x0000000000000000000000000000000000000000000000000000000000000abc";
        for hex in ["0xabc", "abc", "0x0abc", full] {
            let expected: U256 = hex.parse().unwrap();
            assert_eq!(expected, U256::from(0xabc));
            assert_eq!(
                serde_json::from_str::<U256>(&format!("\"{hex}\"")).unwrap(),
                expected
            );
        }
        assert_eq!(
            serde_json::to_string(&U256::from(0xabc)).unwrap(),
            format!("\"{full}\"")
        );

        for invalid in ["", "0x", "0xg", &format!("{full}0")] {
            assert!(invalid.parse::<U256>().is_err(), "{invalid}");
            assert!(
                serde_json::from_str::<U256>(&format!("\"{invalid}\"")).is_err(),
                "{invalid}"
            );
        }
        assert!(serde_json::from_str::<U256>("1").is_err());
    }

    #[test]
    fn bcs_round_trips_as_32_bytes() {
        let value = U256::from(0xabc);
        let bytes = bcs::to_bytes(&value).unwrap();

        assert_eq!(bytes, value.as_slice());
        assert_eq!(bcs::from_bytes::<U256>(&bytes).unwrap(), value);
        assert_eq!(value.as_ref(), value.as_slice());
    }
}
