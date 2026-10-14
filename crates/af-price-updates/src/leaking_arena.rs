//! Append-only arena storage used by the interner helpers.

/// An append-only arena that intentionally leaks allocations.
///
/// Values are stored in heap-allocated chunks that grow as needed. Earlier chunks are never
/// deallocated, so values written to the arena remain valid for the remainder of the program and
/// are returned as `&'static mut T`.
///
/// `min_capacity` controls the first chunk size; subsequent chunks grow geometrically, up to
/// `max_capacity`.
#[derive(Debug)]
pub struct LeakingArena<T: 'static> {
    min_capacity: usize,
    max_capacity: usize,
    last_capacity: usize,
    current: &'static mut [std::mem::MaybeUninit<T>],
}

impl<T> LeakingArena<T> {
    /// Creates an empty arena; no chunk is allocated until the first write.
    pub const fn new(min_capacity: usize, max_capacity: usize) -> Self {
        Self {
            min_capacity,
            max_capacity,
            last_capacity: 0,
            current: &mut [],
        }
    }
    fn allocate_space(&mut self, requested_size: usize) {
        if self.last_capacity < self.min_capacity {
            self.last_capacity = self.min_capacity;
        } else {
            self.last_capacity = self.last_capacity.saturating_mul(2).min(self.max_capacity);
        }
        self.last_capacity = self.last_capacity.max(requested_size.next_power_of_two());
        self.current = Box::leak(Box::new_uninit_slice(self.last_capacity));
    }
    /// Moves `value` into the arena. It is never dropped.
    pub fn write(&mut self, value: T) -> &'static mut T {
        if self.current.is_empty() {
            self.allocate_space(1);
        }
        let place = self.current.split_off_first_mut().expect("allocated above");
        place.write(value)
    }
    /// Copies `slice` into the arena, contiguously.
    pub fn write_copy_of_slice(&mut self, slice: &[T]) -> &'static mut [T]
    where
        T: Copy,
    {
        if slice.len() > self.current.len() {
            self.allocate_space(slice.len());
        }
        let place = self
            .current
            .split_off_mut(..slice.len())
            .expect("allocated above");
        place.write_copy_of_slice(slice)
    }
}

impl LeakingArena<u8> {
    /// Copies `value` into the arena as a string.
    ///
    /// The copied bytes are validated as UTF-8 here, so callers can keep the returned
    /// `&'static str` and never convert from bytes again.
    pub fn write_copy_of_str(&mut self, value: &str) -> &'static mut str {
        std::str::from_utf8_mut(self.write_copy_of_slice(value.as_bytes()))
            .expect("copied from a str")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_slices_into_exactly_fitting_chunk() {
        let mut arena = LeakingArena::new(4, 8);

        let first = arena.write_copy_of_slice(&[1, 2]);
        let second = arena.write_copy_of_slice(&[3, 4]);

        assert_eq!(first, &[1, 2]);
        assert_eq!(second, &[3, 4]);
        assert_eq!(first.as_ptr().wrapping_add(first.len()), second.as_ptr());
    }

    #[test]
    fn writes_empty_values() {
        let mut arena = LeakingArena::<u8>::new(4, 8);

        assert!(arena.write_copy_of_slice(&[]).is_empty());
    }

    #[test]
    fn writes_strings_into_exactly_fitting_chunk() {
        let mut arena = LeakingArena::new(4, 8);

        let first = arena.write_copy_of_str("ab");
        let second = arena.write_copy_of_str("cd");

        assert_eq!(first, "ab");
        assert_eq!(second, "cd");
        assert_eq!(first.as_ptr().wrapping_add(first.len()), second.as_ptr());
    }

    #[test]
    fn writes_empty_and_non_ascii_strings() {
        let mut arena = LeakingArena::new(4, 8);

        assert!(arena.write_copy_of_str("").is_empty());
        assert_eq!(arena.write_copy_of_str("BTC/μUSD"), "BTC/μUSD");
    }
}
