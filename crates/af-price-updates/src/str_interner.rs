//! Process-lifetime interning for UTF-8 strings.

use crate::leaking_arena::LeakingArena;
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::sync::RwLock;

/// Interns strings as canonical references with program-long lifetimes.
///
/// The [`str`] counterpart of [`SliceInterner<u8>`]: equal non-empty strings share the same
/// allocation, allowing returned references to be compared by address. New non-empty values are
/// copied into a [`LeakingArena`], so their storage is intentionally never reclaimed. Lookup and
/// insertion are safe to perform concurrently.
///
/// UTF-8 is validated once, when a new string is copied into the arena. The interner stores the
/// resulting `&'static str`, so lookups return it without converting from bytes again.
///
/// Strings returned by the same [`StrInterner`] are canonical, so wrapper types can implement
/// [`Eq`] and [`Hash`] by comparing and hashing string pointers.
///
/// [`SliceInterner<u8>`]: crate::slice_interner::SliceInterner
#[derive(Debug)]
pub struct StrInterner {
    empty: &'static str,
    inner: RwLock<StrInternerInner>,
}

#[derive(Debug)]
struct StrInternerInner {
    arena: LeakingArena<u8>,
    set: FxHashSet<&'static str>,
}

impl StrInterner {
    /// Creates an empty interner with the given arena chunk capacity bounds.
    ///
    /// The first chunk has at least `min_capacity` bytes. Later chunks grow geometrically up to
    /// `max_capacity`, unless one value requires a larger chunk.
    pub const fn new(min_capacity: usize, max_capacity: usize) -> Self {
        Self {
            empty: "",
            inner: RwLock::new(StrInternerInner::new(min_capacity, max_capacity)),
        }
    }
    /// Returns the canonical reference for `value`.
    ///
    /// Non-empty strings are returned only if they have been interned. Empty strings always return
    /// this interner's canonical empty string.
    pub fn find(&self, value: &str) -> Option<&'static str> {
        if value.is_empty() {
            return Some(self.empty);
        }
        self.inner
            .read()
            .expect("StrInterner lock is poisoned")
            .find(value)
    }
    /// Returns the canonical reference for `value`, interning a copy if needed.
    ///
    /// Empty strings return this interner's canonical empty string without locking or adding a set
    /// entry.
    pub fn get(&self, value: &str) -> &'static str {
        if let Some(value) = self.find(value) {
            return value;
        }
        self.inner
            .write()
            .expect("StrInterner lock is poisoned")
            .get(value)
    }
}

impl StrInternerInner {
    const fn new(min_capacity: usize, max_capacity: usize) -> Self {
        Self {
            arena: LeakingArena::new(min_capacity, max_capacity),
            set: FxHashSet::with_hasher(FxBuildHasher),
        }
    }
    fn find(&self, value: &str) -> Option<&'static str> {
        self.set.get(value).copied()
    }
    fn get(&mut self, value: &str) -> &'static str {
        if let Some(value) = self.find(value) {
            return value;
        }
        let result: &'static str = self.arena.write_copy_of_str(value);
        self.set.insert(result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interns_equal_strings() {
        let interner = StrInterner::new(4, 8);

        let first = interner.get("BTC/USD");
        let second = interner.get(&String::from("BTC/USD"));

        assert_eq!(first.as_ptr(), second.as_ptr());
        assert_eq!(interner.find("BTC/USD"), Some(first));
        assert_eq!(interner.find("ETH/USD"), None);
    }

    #[test]
    fn interns_empty_string() {
        let interner = StrInterner::new(4, 8);

        let found = interner.find("").unwrap();
        let empty = interner.get("");

        assert!(empty.is_empty());
        assert!(std::ptr::addr_eq(found, empty));
        assert_eq!(interner.find(""), Some(empty));
        assert_eq!(interner.inner.read().unwrap().set.len(), 0);
    }

    #[test]
    fn interns_non_ascii_strings() {
        let interner = StrInterner::new(4, 8);

        let value = interner.get("BTC/μUSD");

        assert_eq!(value, "BTC/μUSD");
        assert_eq!(
            interner.find("BTC/μUSD").map(str::as_ptr),
            Some(value.as_ptr())
        );
    }

    #[test]
    #[expect(
        clippy::needless_collect,
        reason = "spawn every thread before joining any"
    )]
    fn interns_concurrently() {
        let interner = std::sync::Arc::new(StrInterner::new(4, 8));
        let handles = (0..4)
            .map(|_| {
                let interner = interner.clone();
                std::thread::spawn(move || interner.get("BTC/USD").as_ptr() as usize)
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
