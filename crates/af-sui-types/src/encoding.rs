//! Encoding types used by Sui.

use base64ct::{Base64, Encoding as _};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

/// Convenience method for decoding base64 bytes the way Sui expects.
pub fn decode_base64_default(value: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    let s = std::str::from_utf8(value.as_ref())
        .map_err(|e| Error::Base64(format!("Invalid UTF-8: {e}")))?;
    Base64::decode_vec(s).map_err(|e| Error::Base64(e.to_string()))
}

/// Convenience method for encoding bytes to base64 the way Sui expects.
pub fn encode_base64_default(value: impl AsRef<[u8]>) -> String {
    base64ct::Base64::encode_string(value.as_ref())
}

// =============================================================================
//  Base64Bcs
// =============================================================================

/// Serialize values with base64-encoded BCS.
///
/// The type serializes a value as a base64 string of its BCS encoding.
/// It works on any type compatible with [`bcs`] for (de)serialization.
pub struct Base64Bcs;

impl<'de, T> DeserializeAs<'de, T> for Base64Bcs
where
    T: for<'a> Deserialize<'a>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Base64::decode_vec(&String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)?;
        bcs::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

impl<T> SerializeAs<T> for Base64Bcs
where
    T: Serialize,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = bcs::to_bytes(source).map_err(serde::ser::Error::custom)?;
        Base64::encode_string(&bytes).serialize(serializer)
    }
}

// =============================================================================
//  Base58
// =============================================================================

/// Base58 encoding format. Can be used with [`serde_with::serde_as`].
pub struct Base58;

impl Base58 {
    pub fn decode(data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        bs58::decode(data)
            .into_vec()
            .map_err(|e| Error::Base58(e.to_string()))
    }

    pub fn encode(data: impl AsRef<[u8]>) -> String {
        bs58::encode(data).into_string()
    }
}

impl<'de, T> DeserializeAs<'de, T> for Base58
where
    T: TryFrom<Vec<u8>>,
    T::Error: std::fmt::Debug,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let value = Self::decode(&s).map_err(serde::de::Error::custom)?;
        let length = value.len();
        value
            .try_into()
            .map_err(|error| serde::de::Error::custom(BytesConversionError { length, error }))
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Converting from a Byte Vector of length {length}: {error:?}")]
pub(crate) struct BytesConversionError<E> {
    pub length: usize,
    pub error: E,
}

impl<T> SerializeAs<T> for Base58
where
    T: AsRef<[u8]>,
{
    fn serialize_as<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Self::encode(value).serialize(serializer)
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Base64: {0}")]
    Base64(String),
    #[error("Base58: {0}")]
    Base58(String),
}
