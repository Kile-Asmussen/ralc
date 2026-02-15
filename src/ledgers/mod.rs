use std::{num::NonZeroU64, ptr::NonNull};

use crate::cookie::CookieJar;

pub mod silo;
#[cfg(any(feature = "parking-lot", test))]
pub mod sync;

pub trait Ledger {
    type Cookies: CookieJar;
    const LIFETIME_NAME: &'static str = "'_";
    fn cookie(&self) -> &Self::Cookies;
    fn reallocation(&self) -> NonZeroU64;

    /// Bump the reallocation count of this ledger
    ///
    /// # Safety of implementation
    /// 1. The value of [`.reallocations()`](Ledger::reallocations) MUST be either
    /// increased or u64::MAX after this function returns.
    unsafe fn bump_impl(&self);

    /// # Safety requirements
    /// 1. This may not be called twice on the same pointer.
    /// 2. The pointer must be convertible to a reference.
    unsafe fn free(_this: NonNull<Self>) {}

    fn read_failure() -> ! {
        panic!("deadlock when acquiring read permissions")
    }

    fn write_failure() -> ! {
        panic!("deadlock when acquiring write permissions")
    }
}

pub trait LedgerExt: Ledger {
    /// Bump the reallocation count of this ledger
    ///
    /// # Safety guarantees
    /// 1. The value of [`.reallocations()`](Ledger::reallocations) is either
    /// increased or u64::MAX after this function returns.
    fn bump(&self) {
        unsafe {
            self.bump_impl();
        }

        // SAFETY:
        // 1. Guaranteed by implementation requirement of bump_impl
    }
}

impl<L: Ledger> LedgerExt for L {}
