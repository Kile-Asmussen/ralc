use std::{
    cell::Cell,
    num::{NonZero, NonZeroU32, NonZeroU64},
    sync::atomic::{AtomicI32, AtomicU16, AtomicU64, Ordering},
};

/// Trait for implementing a reallocation count tracker.
///
/// # Safety requirements
///
/// 1. [`Balance::check`] must return the same value until [`Balance::invalidate`] is called.
/// 2. if [`Balance::exhausted`] returns false, then [`Balance::check`] returns numbers below 2^56
pub unsafe trait Balance: Sized {
    /// Change the output of [`Balance::check`].
    ///
    /// This is used to invalidate ralc pointers' reallocation counts.
    fn invalidate(&self);

    /// Check whether this balance is still good to use or should be discarded
    #[inline]
    fn exhausted(&self) -> bool {
        self.check() >= 0xFF_FFFF_FFFF_FFFF
    }

    /// Get the current reallocation count value.
    fn check(&self) -> u64;
}

// SAFETY:
// 1. Non-mutating
// 2. Trivially true
unsafe impl Balance for Cell<i32> {
    #[inline]
    fn invalidate(&self) {
        self.set(self.get().wrapping_add(1))
    }

    #[inline]
    fn exhausted(&self) -> bool {
        self.get() < 0
    }

    #[inline]
    fn check(&self) -> u64 {
        self.get() as u64
    }
}

// SAFETY:
// 1. Non-mutating
// 2. True by default impl
unsafe impl Balance for Cell<u64> {
    #[inline]
    fn invalidate(&self) {
        self.set(self.get() + 1)
    }

    #[inline]
    fn check(&self) -> u64 {
        self.get()
    }
}

// SAFETY:
// 1. Non-mutating
// 2. True by default implementation
unsafe impl Balance for AtomicU64 {
    #[inline]
    fn invalidate(&self) {
        self.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn check(&self) -> u64 {
        self.load(Ordering::Relaxed)
    }
}

// SAFETY:
// 1. Non-mutating
// 2. Trivially true
unsafe impl Balance for AtomicI32 {
    #[inline]
    fn invalidate(&self) {
        self.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn exhausted(&self) -> bool {
        self.load(Ordering::Relaxed) < 0
    }

    #[inline]
    fn check(&self) -> u64 {
        self.load(Ordering::Relaxed) as u64
    }
}
