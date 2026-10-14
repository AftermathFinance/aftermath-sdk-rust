//! Process-lifetime interning for slices.

use crate::leaking_arena::LeakingArena;
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::{hash::Hash, sync::RwLock};

/// Interns slices as canonical references with program-long lifetimes.
///
/// Equal non-empty slices share the same allocation, allowing returned references to be compared by
/// address. New non-empty values are copied into a [`LeakingArena`], so their storage is
/// intentionally never reclaimed. Lookup and insertion are safe to perform concurrently.
///
/// Slices returned by the same [`SliceInterner`] are canonical, so wrapper types can implement
/// [`Eq`] and [`Hash`] by comparing and hashing slice pointers. Zero-sized element types are
/// rejected because [`std::ptr::addr_eq`] ignores slice length metadata.
#[derive(Debug)]
pub struct SliceInterner<T: 'static> {
    empty: &'static [T],
    inner: RwLock<SliceInternerInner<T>>,
}

#[derive(Debug)]
struct SliceInternerInner<T: 'static> {
    arena: LeakingArena<T>,
    set: FxHashSet<&'static [T]>,
}

impl<T> SliceInterner<T>
where
    T: Copy + Eq + Hash,
{
    /// Creates an empty interner with the given arena chunk capacity bounds.
    ///
    /// The first chunk has at least `min_capacity` elements. Later chunks grow geometrically up to
    /// `max_capacity`, unless one value requires a larger chunk.
    ///
    /// Fails to compile if `T` is zero-sized.
    ///
    /// ```compile_fail
    /// use af_price_updates::slice_interner::SliceInterner;
    ///
    /// let _ = SliceInterner::<()>::new(4, 8);
    /// ```
    pub const fn new(min_capacity: usize, max_capacity: usize) -> Self {
        const {
            assert!(
                std::mem::size_of::<T>() > 0,
                "SliceInterner does not support zero-sized element types",
            );
        };
        Self {
            empty: &[],
            inner: RwLock::new(SliceInternerInner::new(min_capacity, max_capacity)),
        }
    }
    /// Returns the canonical reference for `slice`.
    ///
    /// Non-empty slices are returned only if they have been interned. Empty slices always return
    /// this interner's canonical empty slice.
    pub fn find(&self, slice: &[T]) -> Option<&'static [T]> {
        if slice.is_empty() {
            return Some(self.empty);
        }
        self.inner
            .read()
            .expect("SliceInterner lock is poisoned")
            .find(slice)
    }
    /// Returns the canonical reference for `slice`, interning a copy if needed.
    ///
    /// Empty slices return this interner's canonical empty slice without locking or adding a set
    /// entry.
    pub fn get(&self, slice: &[T]) -> &'static [T] {
        if let Some(slice) = self.find(slice) {
            return slice;
        }
        self.inner
            .write()
            .expect("SliceInterner lock is poisoned")
            .get(slice)
    }
}

impl<T> SliceInternerInner<T>
where
    T: Copy + Eq + Hash,
{
    const fn new(min_capacity: usize, max_capacity: usize) -> Self {
        Self {
            arena: LeakingArena::new(min_capacity, max_capacity),
            set: FxHashSet::with_hasher(FxBuildHasher),
        }
    }
    fn find(&self, slice: &[T]) -> Option<&'static [T]> {
        self.set.get(slice).copied()
    }
    fn get(&mut self, slice: &[T]) -> &'static [T] {
        if let Some(slice) = self.find(slice) {
            return slice;
        }
        let result: &'static [T] = self.arena.write_copy_of_slice(slice);
        self.set.insert(result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interns_equal_slices() {
        let interner = SliceInterner::new(4, 8);

        let first = interner.get(&[1, 2]);
        let second = interner.get(&[1, 2]);

        assert_eq!(first.as_ptr(), second.as_ptr());
        assert_eq!(interner.find(&[1, 2]), Some(first));
        assert_eq!(interner.find(&[3, 4]), None);
    }

    #[test]
    fn interns_empty_slice() {
        let interner = SliceInterner::<u8>::new(4, 8);

        let found = interner.find(&[]).unwrap();
        let empty = interner.get(&[]);

        assert!(empty.is_empty());
        assert!(std::ptr::addr_eq(found, empty));
        assert_eq!(interner.find(&[]), Some(empty));
        assert_eq!(interner.inner.read().unwrap().set.len(), 0);
    }

    #[test]
    #[expect(
        clippy::needless_collect,
        reason = "spawn every thread before joining any"
    )]
    fn interns_concurrently() {
        let interner = std::sync::Arc::new(SliceInterner::new(4, 8));
        let handles = (0..4)
            .map(|_| {
                let interner = interner.clone();
                std::thread::spawn(move || interner.get(&[1, 2]).as_ptr() as usize)
            })
            .collect::<Vec<_>>();
        let pointers = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();

        assert!(pointers.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(interner.inner.read().unwrap().set.len(), 1);
    }
}
