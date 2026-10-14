use crate::{pyth_lazer::PriceFeedId, slice_interner::SliceInterner, sorted_set::SortedSetSlice};
use std::ops::Deref;

static PRICE_FEED_ID_SET_INTERNER: SliceInterner<PriceFeedId> = SliceInterner::new(1024, 2 * 1024);

/// A compact, copyable set of Pyth Lazer price feed IDs.
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
    use crate::sorted_set::SortedSet;
    use std::cmp::Ordering;

    const BTC_USD: PriceFeedId = PriceFeedId(1);
    const USDC_USD: PriceFeedId = PriceFeedId(7);
    const PRICE_FEED_IDS: &[PriceFeedId] = &[
        BTC_USD,
        PriceFeedId(2),
        PriceFeedId(6),
        USDC_USD,
        PriceFeedId(8),
    ];

    fn intern(ids: impl IntoIterator<Item = PriceFeedId>) -> PriceFeedIdSet {
        let ids = ids.into_iter().collect::<SortedSet<_>>();
        PriceFeedIdSet::intern(&ids)
    }

    #[test]
    fn pyth_lazer_price_feed_id_set() {
        let ids0 = PriceFeedIdSet::default();
        assert!(ids0.is_empty());
        assert_eq!(ids0.len(), 0);
        assert_eq!(ids0, intern([]));

        let ids1 = intern([PRICE_FEED_IDS[0]]);
        assert!(!ids1.is_empty());
        assert_eq!(ids1.len(), 1);
        assert_ne!(ids0, ids1);
        assert_eq!(ids1, intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[0]]),);
        assert_eq!(
            ids1,
            intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[0], PRICE_FEED_IDS[0],]),
        );

        let ids2 = intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[1]]);
        assert!(!ids2.is_empty());
        assert_eq!(ids2.len(), 2);
        assert_ne!(ids0, ids2);
        assert_ne!(ids1, ids2);
        assert_eq!(ids2, intern([PRICE_FEED_IDS[1], PRICE_FEED_IDS[0]]),);
        assert_eq!(
            ids2,
            intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[1], PRICE_FEED_IDS[0],]),
        );

        let ids3 = intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[1], PRICE_FEED_IDS[2]]);
        assert!(!ids3.is_empty());
        assert_eq!(ids3.len(), 3);
        assert_ne!(ids0, ids3);
        assert_ne!(ids1, ids3);
        assert_ne!(ids2, ids3);
        assert_eq!(
            ids3,
            intern([PRICE_FEED_IDS[1], PRICE_FEED_IDS[2], PRICE_FEED_IDS[0],]),
        );
        assert_eq!(
            ids3,
            intern([
                PRICE_FEED_IDS[2],
                PRICE_FEED_IDS[0],
                PRICE_FEED_IDS[1],
                PRICE_FEED_IDS[0],
            ])
        );
    }

    #[test]
    fn interns_sorted_set() {
        let ids = SortedSet::from([USDC_USD, BTC_USD, USDC_USD]);

        assert_eq!(
            PriceFeedIdSet::intern(&ids),
            PriceFeedIdSet::from([USDC_USD, BTC_USD, USDC_USD])
        );
    }

    #[test]
    fn find_does_not_intern() {
        // Not used by other tests, which share the interner.
        let ids = SortedSet::from([PriceFeedId(u32::MAX), PriceFeedId(u32::MAX - 1)]);

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
    fn price_feed_id_set_hashes_by_canonical_address() {
        let first = intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[1]]);
        let same = intern([PRICE_FEED_IDS[1], PRICE_FEED_IDS[0]]);
        let different = intern([PRICE_FEED_IDS[0]]);
        let set = std::collections::HashSet::from([first, same, different]);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn empty_price_feed_id_set_is_canonical() {
        let default = PriceFeedIdSet::default();
        let from_iter = intern([]);
        let from_array = PriceFeedIdSet::from([]);
        let parsed = "[]".parse::<SortedSet<PriceFeedId>>().unwrap();
        let parsed = PriceFeedIdSet::intern(&parsed);

        assert!(std::ptr::addr_eq(default.0, from_iter.0));
        assert!(std::ptr::addr_eq(default.0, from_array.0));
        assert!(std::ptr::addr_eq(default.0, parsed.0));

        let set = std::collections::HashSet::from([default, from_iter, from_array, parsed]);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn price_feed_id_set_orders_lexicographically() {
        let first = intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[2]]);
        let same = intern([PRICE_FEED_IDS[2], PRICE_FEED_IDS[0]]);
        let greater = intern([PRICE_FEED_IDS[1]]);

        assert_eq!(first.cmp(&same), Ordering::Equal);
        assert!(first < greater);
    }

    #[test]
    fn display_and_parse_through_owned_set() {
        let empty = PriceFeedIdSet::default();
        assert_eq!(empty.to_string(), "[]");
        let parsed = "[]".parse::<SortedSet<PriceFeedId>>().unwrap();
        assert_eq!(PriceFeedIdSet::intern(&parsed), empty);

        let single = intern([PRICE_FEED_IDS[0]]);
        assert_eq!(single.to_string(), format!("[{}]", PRICE_FEED_IDS[0]));
        let parsed = single
            .to_string()
            .parse::<SortedSet<PriceFeedId>>()
            .unwrap();
        assert_eq!(PriceFeedIdSet::intern(&parsed), single);

        let multi = intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[1], PRICE_FEED_IDS[2]]);
        let s = multi.to_string();
        let parsed = s.parse::<SortedSet<PriceFeedId>>().unwrap();
        assert_eq!(PriceFeedIdSet::intern(&parsed), multi);

        assert!("5,8,20".parse::<SortedSet<PriceFeedId>>().is_err());
        assert!("[5,8,abc]".parse::<SortedSet<PriceFeedId>>().is_err());
    }

    #[test]
    fn serde_uses_owned_set_before_explicit_interning() {
        let owned: SortedSet<PriceFeedId> = serde_json::from_str("[7,1,7]").unwrap();
        let set = PriceFeedIdSet::intern(&owned);

        assert_eq!(set, PriceFeedIdSet::from([BTC_USD, USDC_USD]));
        assert_eq!(serde_json::to_string(&set).unwrap(), "[1,7]");
    }

    #[test]
    fn checks_supersets_through_sorted_set_slice() {
        let superset = intern([PRICE_FEED_IDS[0], PRICE_FEED_IDS[1], PRICE_FEED_IDS[2]]);
        let subset = intern([PRICE_FEED_IDS[2], PRICE_FEED_IDS[0]]);

        assert!(superset.is_superset(&subset));
        assert!(superset.is_superset(&SortedSet::from([PRICE_FEED_IDS[1]])));
        assert!(!subset.is_superset(&superset));
    }
}
