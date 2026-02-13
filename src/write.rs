use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    num::NonZeroU64,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    cookie::{CookieJar, WriteCookie},
    ledgers::Ledger,
    read::ReadRalc,
};

mod _limit_visibility {
    use super::*;

    pub struct WriteRalc<'a, T, L: Ledger> {
        /// ## Safety invariants:
        /// 1. This read token was created from [`Self::ledger`]'s `.cookie()` or `.upgrade()` of one
        cookie: ManuallyDrop<<L::Cookies as CookieJar>::WriteToken<'a>>,
        generation: NonZeroU64,
        /// ## Safety invariants:
        /// 1. Convertible to a mutalbe reference
        ledger: NonNull<L>,
        /// ## Safety invariants:
        /// 1. Convertible to a mutalbe reference
        data: NonNull<ManuallyDrop<T>>,
        _phantom: PhantomData<&'a mut T>,
    }

    impl<'a, T, L: Ledger> WriteRalc<'a, T, L> {
        /// # Safety requirements
        ///
        /// 1. `ledger` must be convertible to a mutable reference
        /// 2. `cookie` must have been created from the cookie jar of `ledger`, or upgraded from one
        /// 3. `data` must be convertible to a mutable reference
        pub(crate) unsafe fn from_parts(
            cookie: <L::Cookies as CookieJar>::WriteToken<'a>,
            generation: NonZeroU64,
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
                generation,
                _phantom: PhantomData,
            }
        }

        pub(crate) fn generation(&self) -> NonZeroU64 {
            self.generation
        }

        /// # Safety guarantees:
        /// 1. The return pointer is exactly the same reference as calling `.ledger()`
        pub(crate) fn ledger_ptr(&self) -> NonNull<L> {
            self.ledger
        }

        /// # Safety guarantees:
        /// 1. The returned pointer is convertible to a mutable reference
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
    }
}

pub use _limit_visibility::WriteRalc;

impl<'a, T, L: Ledger> WriteRalc<'a, T, L> {
    pub(crate) fn ledger(&self) -> &L {
        unsafe {
            // SAFETY:
            // 1. Directly guaranteed
            self.ledger_ptr().as_ref()
        }
    }

    pub fn into_read(mut self) -> ReadRalc<'a, T, L> {
        let generation = self.generation();
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
            ReadRalc::from_parts(cookie.downgrade(), generation, ledger, data)
        }
    }
}

impl<'a, T, L: Ledger> Deref for WriteRalc<'a, T, L> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY:
            // 1. Directly guaranteed
            self.data().as_ref()
        }
    }
}

impl<'a, T, L: Ledger> DerefMut for WriteRalc<'a, T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // SAFETY:
            // 1. Directly guaranteed
            self.data().as_mut()
        }
    }
}

impl<'a, T, L: Ledger> Drop for WriteRalc<'a, T, L> {
    fn drop(&mut self) {
        if self.ledger().generation() != self.generation() {
            unsafe {
                // SAFETY:
                // 1. Directly guaranteed
                let mut slot = *Box::from_raw(self.data().as_ptr());
                // SAFETY:
                // 1. This is `drop`
                ManuallyDrop::drop(&mut slot);
            }
        }
        unsafe {
            // SAFETY:
            // 1. This is `drop`
            ManuallyDrop::drop(self.cookie_mut());
        }
    }
}
