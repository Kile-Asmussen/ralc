use std::{fmt, marker::PhantomData, mem::ManuallyDrop, num::NonZeroU64, ops::Deref, ptr::NonNull};

use crate::{
    cookie::{CookieJar, ReadCookie},
    ledgers::Ledger,
    mut_::RalcMut,
};

mod _limit_visibility {
    use super::*;

    pub struct RalcRef<'a, T, L: Ledger> {
        /// ## Safety invariants:
        /// 1. This read token was created from [`Self::ledger`]'s cookie jar, or cloned or downgrade from one
        cookie: ManuallyDrop<<L::Cookies as CookieJar>::ReadToken<'a>>,
        reallocation: NonZeroU64,
        /// ## Safety invariants:
        /// 1. Convertible to reference
        ledger: NonNull<L>,
        /// ## Safety invariants:
        /// 1. Converitble to reference
        data: NonNull<ManuallyDrop<T>>,
        _phantom: PhantomData<&'a T>,
    }

    impl<'a, T, L: Ledger> RalcRef<'a, T, L> {
        /// # Safety
        ///
        /// 1. `ledger` must be convertible to a reference
        /// 2. `cookie` must have been created from the cookie jar, or cloned or downgrade from one
        /// 3. `data` must be convertible to a reference
        pub(crate) unsafe fn from_parts(
            cookie: <L::Cookies as CookieJar>::ReadToken<'a>,
            reallocation: NonZeroU64,
            ledger: NonNull<L>,
            data: NonNull<ManuallyDrop<T>>,
        ) -> Self {
            Self {
                // SAFETY:
                // 1. Guaranteed by caller
                cookie: ManuallyDrop::new(cookie),
                // SAFETY:
                // 1. Guaranteed by caller
                ledger,
                // SAFETY:
                // 1. Guaranteed by caller
                data,
                reallocation,
                _phantom: PhantomData,
            }
        }

        pub(crate) fn reallocation(&self) -> NonZeroU64 {
            self.reallocation
        }

        pub(crate) fn ledger_ptr(&self) -> NonNull<L> {
            self.ledger
        }

        pub(crate) fn data(&self) -> NonNull<ManuallyDrop<T>> {
            self.data
        }

        pub(crate) fn cookie_mut(
            &mut self,
        ) -> &mut ManuallyDrop<<L::Cookies as CookieJar>::ReadToken<'a>> {
            &mut self.cookie
        }
    }

    impl<'a, T, L: Ledger> Clone for RalcRef<'a, T, L> {
        fn clone(&self) -> Self {
            Self {
                // SAFETY:
                // 1. Upheld by invariant
                cookie: self.cookie.clone(),
                reallocation: self.reallocation.clone(),
                // SAFETY:
                // 1. Upheld by invariant
                ledger: self.ledger.clone(),
                // SAFETY:
                // 1. Upheld by invariant
                data: self.data.clone(),
                _phantom: PhantomData,
            }
        }
    }
}

pub use _limit_visibility::RalcRef;

impl<'a, T, L: Ledger> RalcRef<'a, T, L> {
    pub fn try_into_write(mut self) -> Result<RalcMut<'a, T, L>, Self> {
        let res = unsafe {
            // SAFETY:
            // 1. Call to std::mem::forget on self just below
            self.write_unsafe()
        };
        std::mem::forget(self);
        res
    }

    /// # Safety requirements
    /// 1. If the function returns `Ok` then `self` must not be dropped
    /// 2. If the function retunrs `Ok` then `self` must not be used again
    unsafe fn write_unsafe(&mut self) -> Result<RalcMut<'a, T, L>, Self> {
        let reallocation = self.reallocation();
        let ledger = self.ledger_ptr();
        let data = self.data();
        let cookie = unsafe {
            // SAFETY:
            // 1. Guaranteed by caller (#2)
            ManuallyDrop::take(&mut self.cookie_mut())
        };

        match cookie.try_upgrade() {
            Ok(cookie) => Ok(unsafe {
                // SAFETY:
                // 1.
                RalcMut::from_parts(cookie, reallocation, ledger, data)
            }),
            Err(cookie) => Err(unsafe {
                // SAFETY:
                // 1. Guaranteed by invariant on self
                // 2. Guaranteed by invariant on self
                // 3. Guaranteed by invariant on self
                Self::from_parts(cookie, reallocation, ledger, data)
            }),
        }
    }
}

impl<'a, T, L: Ledger> Deref for RalcRef<'a, T, L> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // 1. Guaranteed by invariant on self
        unsafe { self.data().as_ref() }
    }
}

impl<'a, T, L: Ledger> Drop for RalcRef<'a, T, L> {
    fn drop(&mut self) {
        let write_ralc = unsafe {
            // SAFETY:
            // 1. This is `drop`
            // 2. This is `drop`
            self.write_unsafe()
        };
        match write_ralc {
            Ok(w) => std::mem::drop(w),
            Err(mut r) => unsafe {
                // SAFETY:
                // This is `drop`
                ManuallyDrop::drop(r.cookie_mut());
            },
        }
    }
}

impl<'a, T: fmt::Debug, L: Ledger> fmt::Debug for RalcRef<'a, T, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl<'a, T: fmt::Display, L: Ledger> fmt::Display for RalcRef<'a, T, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.deref(), f)
    }
}
