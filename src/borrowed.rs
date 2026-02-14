use std::{fmt, mem::ManuallyDrop, num::NonZeroU64, ptr::NonNull};

use crate::{RalcMut, RalcRef, cookie::CookieJar, ledgers::Ledger};

mod _limit_visibility {
    use super::*;

    pub struct BorrowRalc<T, L: Ledger> {
        reallocation: NonZeroU64,
        /// # Safety invariant:
        /// 1. Converitble to reference
        ledger: NonNull<L>,
        /// # Safety invariant:
        /// 1. Converitble to reference
        data: NonNull<ManuallyDrop<T>>,
    }

    impl<T, L: Ledger> BorrowRalc<T, L> {
        /// # Safety
        ///
        /// 1. `ledger` must be convertible to a reference
        /// 2. `data` must be have been created from `Box::leak`
        pub(crate) unsafe fn from_parts(
            ledger: NonNull<L>,
            data: NonNull<ManuallyDrop<T>>,
        ) -> Self {
            Self {
                reallocation: unsafe {
                    // SAFETY:
                    // 1. Guaranteed by caller
                    ledger.as_ref()
                }
                .reallocation(),
                ledger,
                data,
            }
        }

        pub fn ledger(&self) -> &L {
            // SAFETY:
            // 1. Guaranteed by invariant
            unsafe { self.ledger.as_ref() }
        }

        pub fn reallocation(&self) -> NonZeroU64 {
            self.reallocation
        }

        /// # Safety guarantees:
        /// 1. The returned pointer is exactly the same as `.ledger()`
        pub(crate) fn ledger_ptr(&self) -> NonNull<L> {
            self.ledger
        }

        /// # Safety guarantees:
        /// 1. The returned pointer is convertible to a reference
        pub(crate) fn data(&self) -> NonNull<ManuallyDrop<T>> {
            self.data
        }
    }

    impl<T, L: Ledger> Clone for BorrowRalc<T, L> {
        fn clone(&self) -> Self {
            Self {
                reallocation: self.reallocation,
                ledger: self.ledger,
                data: self.data,
            }
        }
    }

    impl<T, L: Ledger> Copy for BorrowRalc<T, L> {}
}

pub use _limit_visibility::BorrowRalc;

impl<T, L: Ledger> BorrowRalc<T, L> {
    /// Check if this reference is stale, referencing an allocation that
    /// has already been freed.
    pub fn check(&self) -> bool {
        self.ledger().reallocation() == self.reallocation()
    }

    /// Get a readable reference through this borrow. Wait on access if possible.
    ///
    /// This method panics if the reference is stale or if waiting is not available.
    pub fn read(&self) -> RalcRef<'_, T, L> {
        if !self.check() {
            panic!("Attempt to read through stale borrowed ralc, remember to .check() first!");
        }

        let cookie = self
            .ledger()
            .cookie()
            .read()
            .unwrap_or_else(|| L::read_failure());

        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Cookie is generated just above
            // 3. Guaranteed directly
            RalcRef::from_parts(cookie, self.reallocation(), self.ledger_ptr(), self.data())
        }
    }

    /// Get a writable reference through this borrow. Wait on access if possible.
    ///
    /// This method panics if the reference is stale or if waiting is not available.
    pub fn write(&self) -> RalcMut<'_, T, L> {
        if !self.check() {
            panic!("Attempt to write through stale borrowed ralc, remember to .check() first!");
        }

        let cookie = self
            .ledger()
            .cookie()
            .write()
            .unwrap_or_else(|| L::write_failure());

        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Created just above
            // 3. Guaranteed directly
            RalcMut::from_parts(cookie, self.reallocation(), self.ledger_ptr(), self.data())
        }
    }

    /// Get a readable reference through this borrow. Returns `None` immediately if access
    /// cannot be acquired.
    pub fn try_read(&self) -> Option<RalcRef<'_, T, L>> {
        if !self.check() {
            return None;
        }

        if let Some(cookie) = self.ledger().cookie().try_read() {
            Some(unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                // 2. Cookie is generated just above
                // 3. Guaranteed directly
                RalcRef::from_parts(cookie, self.reallocation(), self.ledger_ptr(), self.data())
            })
        } else {
            None
        }
    }

    /// Get a writable reference through this borrow. Returns `None` immediately if access
    /// cannot be acquired.
    pub fn try_write(&self) -> Option<RalcMut<'_, T, L>> {
        if !self.check() {
            return None;
        }

        if let Some(cookie) = self.ledger().cookie().try_write() {
            Some(unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                // 2. Created just above
                // 3. Guaranteed directly
                RalcMut::from_parts(cookie, self.reallocation(), self.ledger_ptr(), self.data())
            })
        } else {
            None
        }
    }
}

impl<T: fmt::Debug, L: Ledger> fmt::Debug for BorrowRalc<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_tuple("BorrowRalc")
        } else {
            f.debug_tuple(&format!("&{} ralc", L::LIFETIME_NAME))
        }
        .field(
            self.try_read()
                .as_deref()
                .map(|x| x as &dyn fmt::Debug)
                .unwrap_or(&std::any::type_name::<T>() as &dyn fmt::Debug),
        )
        .finish()
    }
}

impl<T: fmt::Display, L: Ledger> fmt::Display for BorrowRalc<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if f.alternate() { " (borrowed)" } else { "" };
        if let Some(r) = self.try_read() {
            write!(f, "{}{}", r, marker)
        } else {
            write!(f, "<unavailable>{}", marker)
        }
    }
}
