use crate::{pyth::PriceFeedId, slice_interner::SliceInterner, sorted_set::SortedSetSlice};
use std::ops::Deref;

static PRICE_FEED_ID_SET_INTERNER: SliceInterner<PriceFeedId> = SliceInterner::new(1024, 2 * 1024);

/// A compact, copyable set of Pyth price feed IDs.
///
/// [`Self::intern`] stores an already sorted and deduplicated set in process-global storage.
/// Equal sets share one canonical slice for the lifetime of the process. Consequently, cloning
/// this type only copies a slice reference, while equality and hashing compare the canonical
/// address in constant time.
///
/// It dereferences to [`SortedSetSlice`], so set queries such as [`SortedSetSlice::is_superset`]
/// work alike on interned, owned and borrowed sets.
///
/// Ordering remains lexicographic by feed ID, and serialization emits the slice contents rather
/// than its process-local identity. Parse or deserialize dynamic input into a
/// [`SortedSet`](crate::sorted_set::SortedSet), then explicitly intern it if the values are trusted
/// and bounded.
///
/// Every distinct set is kept until the process exits, so do not build sets from untrusted input;
/// use [`SortedSet`](crate::sorted_set::SortedSet) for that instead, and [`Self::find`] to look up
/// a set without interning it.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialOrd, serde::Serialize)]
#[serde(transparent)]
pub struct PriceFeedIdSet(&'static [PriceFeedId]);

impl PriceFeedIdSet {
    /// Interns a sorted, deduplicated set of trusted feed IDs.
    ///
    /// Every distinct set remains allocated until the process exits. Keep untrusted or unbounded
    /// input in an owned [`SortedSet`](crate::sorted_set::SortedSet) instead.
    pub fn intern(ids: &SortedSetSlice<PriceFeedId>) -> Self {
        Self(PRICE_FEED_ID_SET_INTERNER.get(ids))
    }
    /// Returns the set of `ids` if an equal set was created before.
    ///
    /// Unlike [`Self::intern`], this never interns `ids`, so it suits untrusted input such as feed
    /// ID sets requested by clients.
    pub fn find(ids: &SortedSetSlice<PriceFeedId>) -> Option<Self> {
        PRICE_FEED_ID_SET_INTERNER.find(ids).map(Self)
    }
}

impl Default for PriceFeedIdSet {
    fn default() -> Self {
        Self(PRICE_FEED_ID_SET_INTERNER.get(&[]))
    }
}

impl Deref for PriceFeedIdSet {
    type Target = SortedSetSlice<PriceFeedId>;
    fn deref(&self) -> &Self::Target {
        SortedSetSlice::from_sorted_slice(self.0)
    }
}

impl AsRef<[PriceFeedId]> for PriceFeedIdSet {
    fn as_ref(&self) -> &[PriceFeedId] {
        self.0
    }
}

impl AsRef<SortedSetSlice<PriceFeedId>> for PriceFeedIdSet {
    fn as_ref(&self) -> &SortedSetSlice<PriceFeedId> {
        self
    }
}

impl<const N: usize> From<[PriceFeedId; N]> for PriceFeedIdSet {
    /// Sorts, deduplicates, and interns a fixed-size array.
    ///
    /// This conversion is intended for bounded, program-controlled values such as array literals.
    /// Use [`Self::intern`] to make interning explicit for dynamically sourced input.
    fn from(mut ids: [PriceFeedId; N]) -> Self {
        Self::intern(SortedSetSlice::sort_dedup(&mut ids))
    }
}

impl IntoIterator for PriceFeedIdSet {
    type Item = PriceFeedId;
    type IntoIter = std::iter::Copied<std::slice::Iter<'static, PriceFeedId>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl PartialEq for PriceFeedIdSet {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(self.0, other.0)
    }
}

impl std::hash::Hash for PriceFeedIdSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl std::fmt::Display for PriceFeedIdSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <SortedSetSlice<PriceFeedId> as std::fmt::Display>::fmt(&**self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{U256, sorted_set::SortedSet};

    fn id(value: u8) -> PriceFeedId {
        PriceFeedId(U256::from(u64::from(value)))
    }

    fn intern(ids: impl IntoIterator<Item = PriceFeedId>) -> PriceFeedIdSet {
        let ids = ids.into_iter().collect::<SortedSet<_>>();
        PriceFeedIdSet::intern(&ids)
    }

    #[test]
    fn canonicalizes_dynamic_sets_before_interning() {
        let first = intern([id(1), id(3), id(1), id(2)]);
        let same = intern([id(2), id(1), id(3)]);

        assert_eq!(first.as_slice(), &[id(1), id(2), id(3)]);
        assert_eq!(first, same);
        assert!(std::ptr::addr_eq(first.0, same.0));
    }

    #[test]
    fn interns_sorted_set() {
        let ids = SortedSet::from([id(3), id(1), id(3)]);

        assert_eq!(
            PriceFeedIdSet::intern(&ids),
            PriceFeedIdSet::from([id(3), id(1), id(3)])
        );
    }

    #[test]
    fn find_does_not_intern() {
        // Not used by other tests, which share the interner.
        let ids = SortedSet::from([
            PriceFeedId(U256::from(u64::MAX)),
            PriceFeedId(U256::from(u64::MAX - 1)),
        ]);

        assert_eq!(PriceFeedIdSet::find(&ids), None);
        assert_eq!(PriceFeedIdSet::find(&ids), None);
        let interned = PriceFeedIdSet::intern(&ids);
        assert_eq!(PriceFeedIdSet::find(&ids), Some(interned));
        assert_eq!(
            PriceFeedIdSet::find(&SortedSet::default()),
            Some(PriceFeedIdSet::default())
        );
    }

    #[test]
    fn empty_set_is_canonical() {
        let default = PriceFeedIdSet::default();
        let from_iter = intern([]);
        let from_array = PriceFeedIdSet::from([]);
        let parsed = "[]".parse::<SortedSet<PriceFeedId>>().unwrap();
        let parsed = PriceFeedIdSet::intern(&parsed);

        assert!(std::ptr::addr_eq(default.0, from_iter.0));
        assert!(std::ptr::addr_eq(default.0, from_array.0));
        assert!(std::ptr::addr_eq(default.0, parsed.0));
    }

    #[test]
    fn hashes_by_canonical_address() {
        let first = intern([id(1), id(2)]);
        let same = intern([id(2), id(1)]);
        let different = intern([id(1)]);
        let set = std::collections::HashSet::from([first, same, different]);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn display_and_parse_round_trip() {
        let set = intern([id(1), id(2)]);
        let serialized = set.to_string();
        let parsed = serialized.parse::<SortedSet<PriceFeedId>>().unwrap();

        assert_eq!(PriceFeedIdSet::intern(&parsed), set);
        assert!("1,2".parse::<SortedSet<PriceFeedId>>().is_err());
        assert!("[1,nope]".parse::<SortedSet<PriceFeedId>>().is_err());
    }

    #[test]
    fn serde_uses_owned_set_before_explicit_interning() {
        let set = PriceFeedIdSet::from([id(1), id(3)]);
        let unsorted = serde_json::to_string(&[id(3), id(1), id(3)]).unwrap();
        let owned = serde_json::from_str::<SortedSet<PriceFeedId>>(&unsorted).unwrap();

        assert_eq!(PriceFeedIdSet::intern(&owned), set);
        assert_eq!(
            serde_json::to_string(&set).unwrap(),
            serde_json::to_string(&owned).unwrap()
        );
    }

    #[test]
    fn checks_supersets_through_sorted_set_slice() {
        let superset = intern([id(1), id(2), id(3)]);
        let subset = intern([id(3), id(1)]);

        assert!(superset.is_superset(&subset));
        assert!(superset.is_superset(&SortedSet::from([id(2)])));
        assert!(!subset.is_superset(&superset));
    }
}
