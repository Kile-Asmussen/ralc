use std::{
    cell::Cell,
    num::{NonZero, NonZeroU32, NonZeroU64},
    sync::atomic::{AtomicU16, AtomicU64, Ordering},
};

/// Trait for implementing a reallocation count tracker.
///
/// # Safety requirements
///
/// 1. [`Balance::check`] must return the same value until [`Balance::invalidate`] is called.
/// 2. [`Balance::check`] returns numbers below 2^56
pub unsafe trait Balance {
    /// Change the output of [`Balance::check`].
    ///
    /// This is used to invalidate ralc pointers' reallocation counts.
    fn invalidate(&self);

    /// Get the current reallocation ocunt value.
    fn check(&self) -> u64;
}

// SAFETY:
// 1. Non-mutating
// 2. 2^64 - 2^56 is basically indistinguishable from 2^64.
//    Assuming it takes a nanosecond to call .invalidate(), it
//    will take 580+ years to do so.
unsafe impl Balance for Cell<u64> {
    #[inline]
    fn invalidate(&self) {
        self.set(self.get() + 1)
    }

    #[inline]
    fn check(&self) -> u64 {
        self.get() & 0xFF_FFFF_FFFF_FFFF
    }
}

// SAFETY:
// 1. Non-mutating
// 2. 2^64 - 2^56 is basically indistinguishable from 2^64.
//    Assuming it takes a nanosecond to call .incr(), it
//    will take 580+ years to do so.
unsafe impl Balance for AtomicU64 {
    #[inline]
    fn invalidate(&self) {
        self.fetch_add(1, Ordering::Release);
    }

    #[inline]
    fn check(&self) -> u64 {
        self.load(Ordering::Acquire) & 0xFF_FFFF_FFFF_FFFF
    }
}
