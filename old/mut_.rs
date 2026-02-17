use std::{
    fmt,
    marker::PhantomData,
    mem::ManuallyDrop,
    num::NonZeroU64,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    cookie::{CookieJar, WriteCookie},
    ledgers::Ledger,
    ref_::RalcRef,
};

mod _limit_visibility {
    use super::*;

    pub struct RalcMut<'a, T, L: Ledger> {
        /// ## Safety invariants:
        /// 1. This read token was created from [`Self::ledger`]'s `.cookie()` or `.upgrade()` of one
        cookie: ManuallyDrop<<L::Cookies as CookieJar>::WriteToken<'a>>,
        reallocation: NonZeroU64,
        /// ## Safety invariants:
        /// 1. Convertible to a mutalbe reference
        ledger: NonNull<L>,
        /// ## Safety invariants:
        /// 1. Convertible to a mutalbe reference and have been created from a box
        data: NonNull<ManuallyDrop<T>>,
        _phantom: PhantomData<&'a mut T>,
    }

    impl<'a, T, L: Ledger> RalcMut<'a, T, L> {
        /// # Safety requirements
        ///
        /// 1. `ledger` must be convertible to a mutable reference
        /// 2. `cookie` must have been created from the cookie jar of `ledger`, or upgraded from one
        /// 3. `data` must be have been created from a box and be convertible to a mutable reference
        pub(crate) unsafe fn from_parts(
            cookie: <L::Cookies as CookieJar>::WriteToken<'a>,
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

        /// # Safety guarantees:
        /// 1. The return pointer is exactly the same reference as calling `.ledger()`
        pub(crate) fn ledger_ptr(&self) -> NonNull<L> {
            self.ledger
        }

        /// # Safety guarantees
        /// 1. The pointer returned is convertible to a reference and was
        ///    created from a Box
        pub(crate) fn data(&self) -> NonNull<ManuallyDrop<T>> {
            self.data
        }

        /// # Safety guarantees:
        /// 1. The cookie is created from `.ledger()`'s cookie jar
        pub(crate) fn cookie_mut(
            &mut self,
        ) -> &mut ManuallyDrop<<L::Cookies as CookieJar>::WriteToken<'a>> {
            &mut self.cookie
        }

        /// # Safety requirements
        /// 1. The owned reference this RalcMut instance came from must not be dropped
        pub(crate) unsafe fn unsafe_take_box(mut self) -> Box<T> {
            let res = unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                let data = Box::from_raw(self.data().cast().as_ptr());
                // SAFETY:
                // 1. forget is called just below
                ManuallyDrop::drop(self.cookie_mut());
                data
            };
            std::mem::forget(self);
            res
        }
    }
}

pub use _limit_visibility::RalcMut;

impl<'a, T, L: Ledger> RalcMut<'a, T, L> {
    pub(crate) fn ledger(&self) -> &L {
        unsafe {
            // SAFETY:
            // 1. Directly guaranteed
            self.ledger_ptr().as_ref()
        }
    }

    pub fn into_read(mut self) -> RalcRef<'a, T, L> {
        let reallocation = self.reallocation();
        let ledger = self.ledger_ptr();
        let data = self.data();
        let cookie = unsafe {
            // SAFETY:
            // 1. forget() is called just below
            ManuallyDrop::take(&mut self.cookie_mut())
        };
        std::mem::forget(self);

        unsafe {
            // SAFETY:
            // 1. Given by safety guarantee
            // 2. Given by safety guarantee
            RalcRef::from_parts(cookie.downgrade(), reallocation, ledger, data)
        }
    }
}

impl<'a, T, L: Ledger> Deref for RalcMut<'a, T, L> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY:
            // 1. Directly guaranteed
            self.data().as_ref()
        }
    }
}

impl<'a, T, L: Ledger> DerefMut for RalcMut<'a, T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // SAFETY:
            // 1. Directly guaranteed
            self.data().as_mut()
        }
    }
}

impl<'a, T, L: Ledger> Drop for RalcMut<'a, T, L> {
    fn drop(&mut self) {
        if self.ledger().reallocation() != self.reallocation() {
            unsafe {
                // SAFETY:
                // 1. Directly guaranteed
                let mut slot = *Box::from_raw(self.data().as_ptr());

                // SAFETY:
                // 1. This is `drop`
                ManuallyDrop::drop(&mut slot);

                // SAFETY:
                // 1. This is `drop`
                // 2. Guaranteed directly
                L::free(self.ledger_ptr());
            }
        }

        unsafe {
            // SAFETY:
            // 1. This is `drop`
            ManuallyDrop::drop(self.cookie_mut());
        }
    }
}

impl<'a, T: fmt::Debug, L: Ledger> fmt::Debug for RalcMut<'a, T, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl<'a, T: fmt::Display, L: Ledger> fmt::Display for RalcMut<'a, T, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.deref(), f)
    }
}
