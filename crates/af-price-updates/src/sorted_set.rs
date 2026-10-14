//! `Vec`-backed, borrowed, and `Arc`-backed sorted sets, plus in-place slice helpers.

use crate::ParseError;
use ref_cast::{RefCastCustom, ref_cast_custom};
use std::{borrow::Borrow, cmp::Ordering, ops::Deref, slice::SliceIndex, str::FromStr, sync::Arc};

/// A sorted, deduplicated `Vec`.
///
/// Converting from a [`Vec`] sorts and deduplicates it in place, reusing its allocation.
/// Converting from an array sorts it in place before allocating capacity for only its unique
/// values. Collecting from an iterator first collects into a `Vec` and then canonicalizes it. The
/// collection dereferences to a [`SortedSetSlice`] and preserves its sorted-and-unique invariant
/// when parsed or deserialized.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize)]
#[serde(transparent)]
pub struct SortedSet<T>(Vec<T>);

impl<T> SortedSet<T> {
    /// Returns the sorted, deduplicated values as a slice.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    /// Returns the values as a borrowed sorted set.
    pub fn as_sorted_set_slice(&self) -> &SortedSetSlice<T> {
        SortedSetSlice::from_sorted_slice(&self.0)
    }
    /// Returns the backing `Vec` without copying its values.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> Default for SortedSet<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> Deref for SortedSet<T> {
    type Target = SortedSetSlice<T>;
    fn deref(&self) -> &Self::Target {
        self.as_sorted_set_slice()
    }
}

impl<T> AsRef<[T]> for SortedSet<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsRef<SortedSetSlice<T>> for SortedSet<T> {
    fn as_ref(&self) -> &SortedSetSlice<T> {
        self.as_sorted_set_slice()
    }
}

impl<T> Borrow<SortedSetSlice<T>> for SortedSet<T> {
    fn borrow(&self) -> &SortedSetSlice<T> {
        self.as_sorted_set_slice()
    }
}

impl<T: Ord> From<Vec<T>> for SortedSet<T> {
    fn from(mut items: Vec<T>) -> Self {
        items.sort_unstable();
        items.dedup();
        Self(items)
    }
}

impl<T: Ord, const N: usize> From<[T; N]> for SortedSet<T> {
    fn from(mut items: [T; N]) -> Self {
        let unique_len = SortedSetSlice::sort_dedup(&mut items).len();
        let mut unique = Vec::with_capacity(unique_len);
        unique.extend(items.into_iter().take(unique_len));
        Self(unique)
    }
}

impl<T: Clone + Ord> From<&[T]> for SortedSet<T> {
    fn from(items: &[T]) -> Self {
        items.to_vec().into()
    }
}

impl<T: Ord> FromIterator<T> for SortedSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        Vec::from_iter(items).into()
    }
}

impl<T> IntoIterator for SortedSet<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'t, T> IntoIterator for &'t SortedSet<T> {
    type Item = &'t T;
    type IntoIter = std::slice::Iter<'t, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: std::fmt::Display> std::fmt::Display for SortedSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <SortedSetSlice<T> as std::fmt::Display>::fmt(self.as_sorted_set_slice(), f)
    }
}

impl<T> FromStr for SortedSet<T>
where
    T: FromStr + Ord,
    T::Err: std::fmt::Display,
{
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_list(s).map(Self::from)
    }
}

impl<'de, T> serde::Deserialize<'de> for SortedSet<T>
where
    T: serde::Deserialize<'de> + Ord,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::deserialize(deserializer).map(Self::from)
    }
}

/// A borrowed view of a sorted, deduplicated slice.
///
/// References to this dynamically sized type carry the lifetime in the reference, just like
/// `&[T]`. Borrowing a [`SortedSet`] or [`ArcSortedSet`] needs no checks. [`TryFrom`] checks a
/// plain slice without sorting or allocating.
///
/// The type exposes no mutable view, so safe code cannot invalidate its strictly increasing
/// order after construction.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, RefCastCustom, serde::Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SortedSetSlice<T>([T]);

impl<T> SortedSetSlice<T> {
    #[ref_cast_custom]
    fn ref_cast(items: &[T]) -> &Self;
    fn index_of<Q>(&self, value: &Q) -> Option<usize>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.0
            .binary_search_by(|item| item.borrow().cmp(value))
            .ok()
    }
    /// Returns the sorted, deduplicated values as a plain slice.
    pub const fn as_slice(&self) -> &[T] {
        &self.0
    }
    /// Returns whether the set contains `value`.
    ///
    /// This uses binary search and accepts borrowed forms of the element type, following
    /// [`std::collections::BTreeSet::contains`].
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.index_of(value).is_some()
    }
    /// Returns whether this set contains every value in `other`.
    ///
    /// Both sets are already sorted, so this performs a linear merge-style scan without
    /// allocating.
    pub fn is_superset(&self, other: &Self) -> bool
    where
        T: Ord,
    {
        if other.is_empty() {
            return true;
        }
        if self.len() < other.len() {
            return false;
        }
        if std::ptr::eq(self, other) {
            return true;
        }
        // Both sets are non-empty here.
        if self.first() > other.first() || self.last() < other.last() {
            return false;
        }

        let mut values = self.iter();
        for other_value in other {
            loop {
                let Some(value) = values.next() else {
                    return false;
                };
                match value.cmp(other_value) {
                    Ordering::Less => {}
                    Ordering::Equal => break,
                    Ordering::Greater => return false,
                }
            }
        }

        true
    }
    /// Checks that `items` are strictly increasing and returns them as a borrowed sorted set.
    ///
    /// This does not sort or allocate. Equal adjacent items are rejected along with out-of-order
    /// items.
    pub fn try_from_slice(items: &[T]) -> Result<&Self, NotStrictlySorted>
    where
        T: Ord,
    {
        if items.is_sorted_by(|a, b| a < b) {
            Ok(Self::from_sorted_slice(items))
        } else {
            Err(NotStrictlySorted)
        }
    }
    /// Sorts and deduplicates `items` in place and returns its unique prefix as a sorted set.
    ///
    /// This does not allocate. Only the returned prefix belongs to the set; the remaining elements
    /// of `items` are left in an unspecified order after duplicate compaction.
    #[must_use]
    pub fn sort_dedup(items: &mut [T]) -> &Self
    where
        T: Ord,
    {
        items.sort_unstable();
        Self::from_sorted_slice(dedup_slice(items))
    }
    /// Returns an invariant-preserving view of `range`, or `None` when it is out of bounds.
    pub fn get_range<R>(&self, range: R) -> Option<&Self>
    where
        R: SliceIndex<[T], Output = [T]>,
    {
        self.0.get(range).map(Self::from_sorted_slice)
    }
    /// Divides the set into two invariant-preserving views at `mid`.
    ///
    /// Panics when `mid` is greater than the length, like [`slice::split_at`].
    pub fn split_at(&self, mid: usize) -> (&Self, &Self) {
        let (left, right) = self.0.split_at(mid);
        (
            Self::from_sorted_slice(left),
            Self::from_sorted_slice(right),
        )
    }
    /// Reinterprets a slice whose sorted-and-unique invariant is already established.
    pub(crate) fn from_sorted_slice(items: &[T]) -> &Self {
        Self::ref_cast(items)
    }
    /// Checks whether `items` contains every set member exactly once and no other values.
    ///
    /// Item order does not matter. A fixed 256-bit stack bitmap detects duplicates without
    /// allocating or sorting; iterators longer than 256 items therefore do not match.
    pub(crate) fn exactly_matches_bounded_iter(
        &self,
        items: impl ExactSizeIterator<Item = T>,
    ) -> bool
    where
        T: Ord,
    {
        const MAX_ITEMS: usize = 256;

        let item_count = items.len();
        if item_count != self.len() || item_count > MAX_ITEMS {
            return false;
        }

        let mut seen = [0_u64; MAX_ITEMS / u64::BITS as usize];
        for item in items {
            let Some(index) = self.index_of(&item) else {
                return false;
            };
            let word = &mut seen[index / u64::BITS as usize];
            let bit = 1_u64 << (index % u64::BITS as usize);
            if *word & bit != 0 {
                return false;
            }
            *word |= bit;
        }
        true
    }
}

impl<T> Deref for SortedSetSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> AsRef<[T]> for SortedSetSlice<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsRef<Self> for SortedSetSlice<T> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<'t, T: Ord> TryFrom<&'t [T]> for &'t SortedSetSlice<T> {
    type Error = NotStrictlySorted;
    fn try_from(items: &'t [T]) -> Result<Self, Self::Error> {
        SortedSetSlice::try_from_slice(items)
    }
}

impl<T: Clone> ToOwned for SortedSetSlice<T> {
    type Owned = SortedSet<T>;
    fn to_owned(&self) -> Self::Owned {
        SortedSet(self.0.to_vec())
    }
}

impl<T: Clone> From<&SortedSetSlice<T>> for SortedSet<T> {
    fn from(items: &SortedSetSlice<T>) -> Self {
        items.to_owned()
    }
}

impl<'t, T> IntoIterator for &'t SortedSetSlice<T> {
    type Item = &'t T;
    type IntoIter = std::slice::Iter<'t, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: std::fmt::Display> std::fmt::Display for SortedSetSlice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        let mut items = self.iter();
        if let Some(first) = items.next() {
            write!(f, "{first}")?;
            for item in items {
                write!(f, ",{item}")?;
            }
        }
        f.write_str("]")
    }
}

/// A cheaply cloneable, shared sorted set backed by an [`Arc`] slice.
///
/// Conversion from a [`Vec`], array, slice, iterator, or deserialized sequence sorts and
/// deduplicates the values. Conversion from a [`SortedSet`] or [`SortedSetSlice`] reuses their
/// established invariant without sorting again.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArcSortedSet<T>(Arc<[T]>);

impl<T> ArcSortedSet<T> {
    /// Returns the sorted, deduplicated values as a plain slice.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    /// Returns the values as a borrowed sorted set.
    pub fn as_sorted_set_slice(&self) -> &SortedSetSlice<T> {
        SortedSetSlice::from_sorted_slice(&self.0)
    }
    /// Returns the backing `Arc` without copying its values.
    pub fn into_arc(self) -> Arc<[T]> {
        self.0
    }
}

impl<T> Default for ArcSortedSet<T> {
    fn default() -> Self {
        Self(Arc::default())
    }
}

impl<T> Deref for ArcSortedSet<T> {
    type Target = SortedSetSlice<T>;
    fn deref(&self) -> &Self::Target {
        self.as_sorted_set_slice()
    }
}

impl<T> AsRef<[T]> for ArcSortedSet<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsRef<SortedSetSlice<T>> for ArcSortedSet<T> {
    fn as_ref(&self) -> &SortedSetSlice<T> {
        self.as_sorted_set_slice()
    }
}

impl<T> Borrow<SortedSetSlice<T>> for ArcSortedSet<T> {
    fn borrow(&self) -> &SortedSetSlice<T> {
        self.as_sorted_set_slice()
    }
}

impl<T> From<SortedSet<T>> for ArcSortedSet<T> {
    fn from(items: SortedSet<T>) -> Self {
        Self(items.0.into())
    }
}

impl<T: Clone> From<&SortedSet<T>> for ArcSortedSet<T> {
    fn from(items: &SortedSet<T>) -> Self {
        Self::from(items.as_sorted_set_slice())
    }
}

impl<T: Ord> From<Vec<T>> for ArcSortedSet<T> {
    fn from(items: Vec<T>) -> Self {
        SortedSet::from(items).into()
    }
}

impl<T: Ord, const N: usize> From<[T; N]> for ArcSortedSet<T> {
    fn from(items: [T; N]) -> Self {
        SortedSet::from(items).into()
    }
}

impl<T: Clone + Ord> From<&[T]> for ArcSortedSet<T> {
    fn from(items: &[T]) -> Self {
        SortedSet::from(items).into()
    }
}

impl<T: Clone> From<&SortedSetSlice<T>> for ArcSortedSet<T> {
    fn from(items: &SortedSetSlice<T>) -> Self {
        Self(Arc::from(items.as_slice()))
    }
}

impl<T: Ord> FromIterator<T> for ArcSortedSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        SortedSet::from_iter(items).into()
    }
}

impl<'t, T> IntoIterator for &'t ArcSortedSet<T> {
    type Item = &'t T;
    type IntoIter = std::slice::Iter<'t, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: std::fmt::Display> std::fmt::Display for ArcSortedSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <SortedSetSlice<T> as std::fmt::Display>::fmt(self.as_sorted_set_slice(), f)
    }
}

impl<T> FromStr for ArcSortedSet<T>
where
    T: FromStr + Ord,
    T::Err: std::fmt::Display,
{
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_list(s).map(Self::from)
    }
}

impl<T: serde::Serialize> serde::Serialize for ArcSortedSet<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

impl<'de, T> serde::Deserialize<'de> for ArcSortedSet<T>
where
    T: serde::Deserialize<'de> + Ord,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::deserialize(deserializer).map(Self::from)
    }
}

/// The error returned when converting a slice that is not strictly sorted into a
/// [`SortedSetSlice`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NotStrictlySorted;

impl std::fmt::Display for NotStrictlySorted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("slice is not strictly sorted")
    }
}

impl std::error::Error for NotStrictlySorted {}

/// Parses the `[a,b,c]` form written by the sorted-set `Display` implementations, ignoring
/// whitespace around items.
///
/// The items are returned in the order written, neither sorted nor deduplicated.
fn parse_list<T>(s: &str) -> Result<Vec<T>, ParseError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let inner = s
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| ParseError::new(format!("expected [id,...]: {s}")))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut items = Vec::with_capacity(inner.matches(',').count() + 1);
    for item in inner.split(',') {
        items.push(
            item.trim()
                .parse()
                .map_err(|err: T::Err| ParseError::new(err.to_string()))?,
        );
    }
    Ok(items)
}

/// Deduplicates consecutive equal elements in a slice in place and returns its unique runs.
fn dedup_slice<T: PartialEq>(slice: &mut [T]) -> &[T] {
    if slice.len() < 2 {
        return slice;
    }
    let mut i = 0;
    for j in 1..slice.len() {
        if slice[i] != slice[j] {
            i += 1;
            slice.swap(i, j);
        }
    }
    &slice[..=i]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_slice_keeps_one_item_per_run() {
        assert!(dedup_slice::<i32>(&mut []).is_empty());
        assert_eq!(dedup_slice(&mut [1]), &[1]);
        assert_eq!(dedup_slice(&mut [1, 1]), &[1]);
        assert_eq!(dedup_slice(&mut [1, 2]), &[1, 2]);
        assert_eq!(dedup_slice(&mut [2, 1]), &[2, 1]);
        assert_eq!(dedup_slice(&mut [1, 1, 2]), &[1, 2]);
        assert_eq!(dedup_slice(&mut [1, 2, 2]), &[1, 2]);
        assert_eq!(dedup_slice(&mut [1, 1, 2, 2]), &[1, 2]);
        assert_eq!(dedup_slice(&mut [1, 2, 1]), &[1, 2, 1]);
        assert_eq!(dedup_slice(&mut [1, 2, 2, 1]), &[1, 2, 1]);
        assert_eq!(dedup_slice(&mut [1, 1, 1, 2, 1, 1, 1]), &[1, 2, 1]);
        assert_eq!(dedup_slice(&mut [1, 2, 2, 1, 1]), &[1, 2, 1]);
        assert_eq!(dedup_slice(&mut [1, 1, 3, 3, 3, 1, 1, 3]), &[1, 3, 1, 3]);
    }

    #[test]
    fn sorted_set_slice_sorts_and_deduplicates() {
        assert!(SortedSetSlice::<i32>::sort_dedup(&mut []).is_empty());
        assert_eq!(SortedSetSlice::sort_dedup(&mut [1]).as_slice(), &[1]);
        assert_eq!(SortedSetSlice::sort_dedup(&mut [1, 1]).as_slice(), &[1]);
        assert_eq!(SortedSetSlice::sort_dedup(&mut [1, 2]).as_slice(), &[1, 2]);
        assert_eq!(SortedSetSlice::sort_dedup(&mut [2, 1]).as_slice(), &[1, 2]);
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [1, 1, 2]).as_slice(),
            &[1, 2]
        );
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [1, 2, 2]).as_slice(),
            &[1, 2]
        );
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [1, 1, 2, 2]).as_slice(),
            &[1, 2]
        );
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [1, 2, 1]).as_slice(),
            &[1, 2]
        );
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [1, 2, 2, 1]).as_slice(),
            &[1, 2]
        );
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [1, 1, 1, 2, 1, 1, 1]).as_slice(),
            &[1, 2]
        );
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [1, 2, 2, 1, 1]).as_slice(),
            &[1, 2]
        );
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [1, 1, 3, 3, 3, 1, 1, 3]).as_slice(),
            &[1, 3]
        );
        assert_eq!(
            SortedSetSlice::sort_dedup(&mut [2, 3, 1, 2, 3, 1, 3, 2, 1, 2, 3, 2, 1, 3, 2,])
                .as_slice(),
            &[1, 2, 3],
        );
    }

    #[test]
    fn sorted_set_reuses_allocation() {
        let items = vec![3, 1, 3, 2];
        let ptr = items.as_ptr();

        let set = SortedSet::from(items);

        assert_eq!(set.as_slice(), &[1, 2, 3]);
        assert_eq!(set.as_ptr(), ptr);
        assert_eq!(set, [2, 3, 1].into_iter().collect());
        assert_eq!(set.clone().into_vec(), vec![1, 2, 3]);
        assert_eq!(set.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn constructs_from_arrays_and_slices() {
        let items = [
            String::from("three"),
            String::from("one"),
            String::from("three"),
        ];

        assert_eq!(
            SortedSet::from(items.clone()),
            SortedSet::from(items.as_slice())
        );
        assert_eq!(
            SortedSet::from(items).into_vec(),
            vec![String::from("one"), String::from("three")]
        );
    }

    #[test]
    fn displays_and_parses_lists() {
        assert_eq!(SortedSet::<u32>::default().to_string(), "[]");
        assert_eq!(SortedSet::from(vec![7, 3, 7]).to_string(), "[3,7]");

        assert_eq!(parse_list::<u32>("[]").unwrap(), Vec::<u32>::new());
        assert_eq!(parse_list::<u32>("[ ]").unwrap(), Vec::<u32>::new());
        assert_eq!(parse_list::<u32>("[7,3,7]").unwrap(), [7, 3, 7]);
        assert_eq!(parse_list::<u32>("[ 7, 3 ]").unwrap(), [7, 3]);
        assert_eq!("[7,3,7]".parse(), Ok(SortedSet::from(vec![3, 7])));

        for invalid in ["7,3", "[7,3", "7,3]", "[7,,3]", "[7,x]", "[-1]"] {
            assert!(parse_list::<u32>(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn sorted_set_serde() {
        let set: SortedSet<u32> = serde_json::from_str("[5,2,5]").unwrap();

        assert_eq!(set.as_slice(), &[2, 5]);
        assert_eq!(serde_json::to_string(&set).unwrap(), "[2,5]");
    }

    #[test]
    fn sorted_set_slice_checks_plain_slices() {
        assert_eq!(SortedSetSlice::<u32>::try_from_slice(&[]).unwrap().len(), 0);
        assert_eq!(
            SortedSetSlice::try_from_slice(&[1, 2, 3])
                .unwrap()
                .as_slice(),
            &[1, 2, 3]
        );
        assert_eq!(
            SortedSetSlice::try_from_slice(&[2, 1]),
            Err(NotStrictlySorted)
        );
        assert_eq!(
            SortedSetSlice::try_from_slice(&[1, 1]),
            Err(NotStrictlySorted)
        );

        let owned = SortedSet::from([3, 1, 3]);
        let borrowed = owned.as_sorted_set_slice();
        assert_eq!(borrowed.as_slice(), &[1, 3]);
        assert_eq!(borrowed.to_string(), "[1,3]");
        assert_eq!(SortedSet::from(borrowed), owned);

        let set = SortedSet::from([5, 3, 1]);
        assert_eq!(set.get_range(1..).unwrap().as_slice(), &[3, 5]);
        assert!(set.get_range(4..).is_none());
        let (left, right) = set.split_at(1);
        assert_eq!(left.as_slice(), &[1]);
        assert_eq!(right.as_slice(), &[3, 5]);
    }

    #[test]
    fn arc_sorted_set_is_canonical_and_cheap_to_clone() {
        let set = ArcSortedSet::from(vec![3, 1, 3, 2]);
        let clone = set.clone();

        assert_eq!(set.as_slice(), &[1, 2, 3]);
        assert!(Arc::ptr_eq(&set.0, &clone.0));
        let borrowed: &SortedSetSlice<_> = &set;
        assert_eq!(borrowed.as_slice(), &[1, 2, 3]);
        assert_eq!(
            set.as_sorted_set_slice().to_owned(),
            SortedSet::from([1, 2, 3])
        );
        assert_eq!(clone.into_arc().as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn arc_sorted_set_displays_parses_and_serializes() {
        let parsed: ArcSortedSet<u32> = "[7,3,7]".parse().unwrap();
        assert_eq!(parsed.as_slice(), &[3, 7]);
        assert_eq!(parsed.to_string(), "[3,7]");
        assert_eq!(serde_json::to_string(&parsed).unwrap(), "[3,7]");

        let deserialized: ArcSortedSet<u32> = serde_json::from_str("[5,2,5]").unwrap();
        assert_eq!(deserialized.as_slice(), &[2, 5]);
    }

    #[test]
    fn exact_set_match_rejects_duplicates_without_allocating() {
        let sorted_set = SortedSetSlice::try_from_slice(&[1, 2, 3]).unwrap();

        assert!(sorted_set.exactly_matches_bounded_iter([3, 1, 2].into_iter()));
        assert!(!sorted_set.exactly_matches_bounded_iter([1, 1, 3].into_iter()));
        assert!(!sorted_set.exactly_matches_bounded_iter([1, 2].into_iter()));
        assert!(!sorted_set.exactly_matches_bounded_iter([1, 2, 4].into_iter()));
    }

    #[test]
    fn sorted_set_contains_borrowed_values() {
        let set = SortedSet::from([
            String::from("three"),
            String::from("one"),
            String::from("two"),
        ]);

        assert!(set.contains("one"));
        assert!(set.contains("two"));
        assert!(!set.contains("four"));
    }

    #[test]
    fn sorted_set_slice_checks_supersets() {
        let empty = SortedSet::<u32>::default();
        let subset = SortedSet::from([2, 4]);
        let superset = SortedSet::from([1, 2, 3, 4, 5]);
        let missing_middle = SortedSet::from([1, 2, 4, 5]);
        let outside_bounds = SortedSet::from([0, 2]);

        assert!(empty.is_superset(&empty));
        assert!(superset.is_superset(&empty));
        assert!(superset.is_superset(&subset));
        assert!(superset.is_superset(&superset));
        assert!(!empty.is_superset(&subset));
        assert!(!subset.is_superset(&superset));
        assert!(!missing_middle.is_superset(&SortedSet::from([2, 3, 4])));
        assert!(!superset.is_superset(&outside_bounds));
    }

    #[test]
    fn is_superset_matches_membership_for_all_small_sets() {
        const UNIVERSE: [u32; 5] = [1, 2, 6, 7, 8];
        let set = |mask: u8| {
            UNIVERSE
                .iter()
                .enumerate()
                .filter_map(|(index, &value)| (mask & (1 << index) != 0).then_some(value))
                .collect::<SortedSet<_>>()
        };

        for superset_mask in 0..32 {
            let superset = set(superset_mask);
            for subset_mask in 0..32 {
                let subset = set(subset_mask);
                let expected = subset.iter().all(|value| superset.contains(value));

                assert_eq!(
                    superset.is_superset(&subset),
                    expected,
                    "superset={superset}, subset={subset}",
                );
            }
        }
    }

    #[test]
    fn is_superset_accepts_equal_sets_checked_slices_and_views() {
        let set = SortedSet::from([1, 2, 3]);
        // This prefix has the same data address as `set`, but different slice metadata.
        let prefix = set.get_range(..2).unwrap();

        assert!(set.is_superset(&SortedSet::from([3, 1, 2])));
        assert!(set.is_superset(SortedSetSlice::try_from_slice(&[1, 2, 3]).unwrap()));
        assert!(set.is_superset(&ArcSortedSet::from([3, 2])));
        assert!(set.is_superset(prefix));
        assert!(!prefix.is_superset(&set));
        assert!(!set.is_superset(&SortedSet::from([1, 2, 4])));
    }

    #[test]
    fn exact_set_match_enforces_bitmap_capacity() {
        let maximum = (0_u16..256).collect::<SortedSet<_>>();
        let too_large = (0_u16..257).collect::<SortedSet<_>>();

        assert!(maximum.exactly_matches_bounded_iter((0_u16..256).rev()));
        assert!(!too_large.exactly_matches_bounded_iter((0_u16..257).rev()));
    }
}
