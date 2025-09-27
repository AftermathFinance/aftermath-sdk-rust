use std::fmt::Display;
use std::str::FromStr;

use af_sui_types::GasCostSummary;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, DisplayFromStr, SerializeAs, serde_as};

// =============================================================================
//  BigInt
// =============================================================================

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
pub struct BigInt<T>(#[serde_as(as = "serde_with::DisplayFromStr")] T)
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display;

impl<T> BigInt<T>
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> SerializeAs<T> for BigInt<T>
where
    T: Display + FromStr + Copy,
    <T as FromStr>::Err: Display,
{
    fn serialize_as<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Self(*value).serialize(serializer)
    }
}

impl<'de, T> DeserializeAs<'de, T> for BigInt<T>
where
    T: Display + FromStr + Copy,
    <T as FromStr>::Err: Display,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(*Self::deserialize(deserializer)?)
    }
}

impl<T> From<T> for BigInt<T>
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display,
{
    fn from(v: T) -> Self {
        Self(v)
    }
}

impl<T> std::ops::Deref for BigInt<T>
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Display for BigInt<T>
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
//  Base64orBase58
// =============================================================================

/// Always serialize as base64, but deserialize from either Base64 or Base58
pub(crate) struct Base64orBase58;

impl<T> SerializeAs<T> for Base64orBase58
where
    T: AsRef<[u8]>,
{
    fn serialize_as<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded_string = af_sui_types::encode_base64_default(value);
        encoded_string.serialize(serializer)
    }
}

impl<'de> DeserializeAs<'de, Vec<u8>> for Base64orBase58 {
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        af_sui_types::decode_base64_default(&s)
            .or_else(|_| af_sui_types::encoding::Base58::decode(&s))
            .map_err(|_| serde::de::Error::custom("Deserialization failed"))
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GasCostSummaryJson {
    #[serde_as(as = "DisplayFromStr")]
    computation_cost: u64,
    #[serde_as(as = "DisplayFromStr")]
    storage_cost: u64,
    #[serde_as(as = "DisplayFromStr")]
    storage_rebate: u64,
    #[serde_as(as = "DisplayFromStr")]
    non_refundable_storage_fee: u64,
}

impl From<GasCostSummaryJson> for GasCostSummary {
    fn from(
        GasCostSummaryJson {
            computation_cost,
            storage_cost,
            storage_rebate,
            non_refundable_storage_fee,
        }: GasCostSummaryJson,
    ) -> Self {
        Self {
            computation_cost,
            storage_cost,
            storage_rebate,
            non_refundable_storage_fee,
        }
    }
}

impl From<GasCostSummary> for GasCostSummaryJson {
    fn from(
        GasCostSummary {
            computation_cost,
            storage_cost,
            storage_rebate,
            non_refundable_storage_fee,
        }: GasCostSummary,
    ) -> Self {
        Self {
            computation_cost,
            storage_cost,
            storage_rebate,
            non_refundable_storage_fee,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[serde_as]
    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    struct Bcs {
        #[serde_as(as = "Base64orBase58")]
        bcs: Vec<u8>,
    }

    #[test]
    fn new_bcs_format() {
        let bytes = vec![0, 1, 2, 3, 4];
        let untagged_base58 = r#"{"bcs":"12VfUX"}"#;
        let tagged_base58 = r#"{"bcsEncoding":"base58","bcs":"12VfUX"}"#;
        let tagged_base64 = r#"{"bcsEncoding":"base64","bcs":"AAECAwQ="}"#;

        println!(
            "{}",
            serde_json::to_string(&Bcs { bcs: bytes.clone() }).unwrap()
        );

        assert_eq!(
            bytes,
            serde_json::from_str::<Bcs>(untagged_base58).unwrap().bcs
        );
        assert_eq!(
            bytes,
            serde_json::from_str::<Bcs>(tagged_base58).unwrap().bcs
        );
        assert_eq!(
            bytes,
            serde_json::from_str::<Bcs>(tagged_base64).unwrap().bcs
        );

        // Roundtrip base64
        let name = serde_json::from_str::<Bcs>(tagged_base64).unwrap();
        let json = serde_json::to_string(&name).unwrap();
        let from_json = serde_json::from_str::<Bcs>(&json).unwrap();
        assert_eq!(name, from_json);

        // Roundtrip base58
        let name = serde_json::from_str::<Bcs>(tagged_base58).unwrap();
        let json = serde_json::to_string(&name).unwrap();
        let from_json = serde_json::from_str::<Bcs>(&json).unwrap();
        assert_eq!(name, from_json);
    }
}
