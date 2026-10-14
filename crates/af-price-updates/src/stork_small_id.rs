use crate::str_interner::StrInterner;
use serde::{Deserialize, Serialize};

static ASSET_ID_INTERNER: StrInterner = StrInterner::new(1024, 2 * 1024);

/// Copyable and smaller version of Stork asset id.
///
/// Equal asset ids share one interned string, so equality and hashing compare addresses in constant
/// time. Every distinct asset id is kept until the process exits, so do not create them from
/// untrusted input.
///
/// Deserialization never interns: it accepts only the empty ID and non-empty IDs previously
/// introduced through [`Self::intern`]. Use [`Self::find`] for the same lookup without serde.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialOrd, Serialize)]
pub struct SmallAssetId(&'static str);

impl SmallAssetId {
    /// Interns a trusted, bounded asset ID for the lifetime of the process.
    ///
    /// Every distinct string remains allocated until the process exits. Use [`Self::find`] or an
    /// owned string for untrusted or unbounded input.
    pub fn intern(s: &str) -> Self {
        Self(ASSET_ID_INTERNER.get(s))
    }
    /// Returns the asset id `s` if it was created before.
    ///
    /// Unlike [`Self::intern`], this never interns `s`, so it suits untrusted input such as asset
    /// IDs requested by clients.
    pub fn find(s: &str) -> Option<Self> {
        ASSET_ID_INTERNER.find(s).map(Self)
    }
}

impl Default for SmallAssetId {
    fn default() -> Self {
        Self(ASSET_ID_INTERNER.get(""))
    }
}

impl AsRef<str> for SmallAssetId {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl std::ops::Deref for SmallAssetId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'de> Deserialize<'de> for SmallAssetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct AssetIdVisitor;

        impl serde::de::Visitor<'_> for AssetIdVisitor {
            type Value = SmallAssetId;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a previously interned Stork asset ID")
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                SmallAssetId::find(s)
                    .ok_or_else(|| E::custom(format_args!("unknown Stork asset ID: {s}")))
            }
        }

        deserializer.deserialize_str(AssetIdVisitor)
    }
}

impl std::fmt::Display for SmallAssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for SmallAssetId {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(self.0, other.0)
    }
}

impl std::hash::Hash for SmallAssetId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copyable_asset_id() {
        let a1 = SmallAssetId::intern("BTCUSD");
        let a2 = SmallAssetId::intern("BTCUSD");
        assert_eq!(a1, a2);
        assert_eq!(a1.0.as_ptr(), a2.0.as_ptr());
        let a3 = SmallAssetId::intern("ETCUSD");
        assert_ne!(a1, a3);
    }
    #[test]
    fn find_does_not_intern() {
        // Not used by other tests, which share the interner.
        const ASSET_ID: &str = "FINDTESTUSD";

        assert_eq!(SmallAssetId::find(ASSET_ID), None);
        assert_eq!(SmallAssetId::find(ASSET_ID), None);
        let interned = SmallAssetId::intern(ASSET_ID);
        assert_eq!(SmallAssetId::find(ASSET_ID), Some(interned));
        assert_eq!(SmallAssetId::find(""), Some(SmallAssetId::default()));
    }
    #[test]
    fn deserialization_rejects_unknown_id_without_interning_it() {
        // Not used by other tests, which share the interner.
        const ASSET_ID: &str = "UNKNOWNDESERIALIZATIONTESTUSD";

        assert_eq!(SmallAssetId::find(ASSET_ID), None);
        assert!(serde_json::from_str::<SmallAssetId>(&format!(r#""{ASSET_ID}""#)).is_err());
        assert_eq!(SmallAssetId::find(ASSET_ID), None);
    }
    #[test]
    fn small_asset_id_hashes_by_canonical_address() {
        let first = SmallAssetId::intern("BTCUSD");
        let same = SmallAssetId::intern(&String::from("BTCUSD"));
        let different = SmallAssetId::intern("ETHUSD");
        let set = std::collections::HashSet::from([first, same, different]);

        assert_eq!(set.len(), 2);
    }
    #[test]
    fn empty_asset_id_is_canonical() {
        let default = SmallAssetId::default();
        let from_str = SmallAssetId::intern("");
        let deserialized = serde_json::from_str::<SmallAssetId>(r#""""#).unwrap();

        assert!(std::ptr::addr_eq(default.0, from_str.0));
        assert!(std::ptr::addr_eq(default.0, deserialized.0));

        let set = std::collections::HashSet::from([default, from_str, deserialized]);
        assert_eq!(set.len(), 1);
    }
    #[test]
    fn copyable_asset_id_serde() {
        #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
        struct Data {
            assets: Vec<SmallAssetId>,
        }
        let data = Data {
            assets: vec![
                SmallAssetId::intern("BTCUSD"),
                SmallAssetId::intern("ETHUSD"),
            ],
        };
        const EXPECTED: &str = r#"{"assets":["BTCUSD","ETHUSD"]}"#;
        let s = serde_json::to_string(&data).unwrap();
        assert_eq!(s, EXPECTED);
        let data2: Data = serde_json::from_str(EXPECTED).unwrap();
        assert_eq!(data, data2);
    }
    #[test]
    fn deserializes_transient_and_owned_strings() {
        // None of these lend a `&str` borrowed from the input.
        let expected = SmallAssetId::intern("BTCUSD");

        assert_eq!(
            serde_json::from_str::<SmallAssetId>(r#""BTC\u0055SD""#).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::from_reader::<_, SmallAssetId>(&br#""BTCUSD""#[..]).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::from_value::<SmallAssetId>(serde_json::Value::String("BTCUSD".into()))
                .unwrap(),
            expected
        );
    }
}
