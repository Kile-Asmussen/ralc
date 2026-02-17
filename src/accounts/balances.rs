use std::{
    cell::Cell,
    num::{NonZero, NonZeroU32, NonZeroU64},
    sync::atomic::{AtomicU16, AtomicU64, Ordering},
};

/// # Safety requirements:
/// ```rust
/// # use racl::accounts::{counts::*, counters::*};
/// # fn test<Impl : Counter>() {
/// // given Impl : Counter
///
/// // 1.
/// assert!(!Impl::INIT.get().is_none());
///
/// // 2.
/// let counter : Impl;
/// # counter = Impl::INIT;
/// # for _ in 0 .. 1_000_000 {
/// let n1 = counter.get();
/// let n2 = counter.get();
/// # counter.incr();
/// assert!(n1 == n2);
/// # }
///
/// // 3.
/// let counter : Impl;
/// # counter = Impl::INIT;
/// # for _ in 0 .. 1_000_000 {
/// let n1 = counter.get();
/// counter.incr();
/// let n2 = counter.get();
/// assert!(n1 < n2);
///
/// // 4.
/// # counter = Impl::INIT;
/// assert!(counter.get() < 0x100_0000_0000_0000);
///
/// // 5.
/// if counter.get().is_none() {
/// # for _ in 0 .. 1_000_000 {
/// # counter.incr();
///   counter.get().is_none()
/// # }
/// }
/// # }
/// # }
/// # test::<CellCounter>();
/// # test::<AtomicCounter>();
/// ```
pub unsafe trait Balance {
    const INIT: Self;
    fn invalidate(&self);
    fn check(&self) -> u64;
}

// SAFETY:
// 1. Doctests pass
// 2. Doctests pass
// 3. Doctests pass
// 4. Bitmask
// 5. 2^64 - 2^56 is basically indistinguishable from 2^64.
//    Assuming it takes a nanosecond to call .incr(), it
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

    const INIT: Self = Cell::new(1);
}

// SAFETY:
// 1. Doctests pass
// 2. Doctests pass
// 3. Doctests pass
// 4. Bitmask
// 5. 2^64 - 2^56 is basically indistinguishable from 2^64.
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

    const INIT: Self = AtomicU64::new(0);
}
