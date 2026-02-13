use std::{marker::PhantomData, mem::ManuallyDrop, num::NonZeroU64, ops::Deref, ptr::NonNull};

use crate::{
    cookie::{CookieJar, ReadCookie},
    ledgers::Ledger,
    write::WriteRalc,
};

mod _limit_visibility {
    use super::*;

    pub struct ReadRalc<'a, T, L: Ledger> {
        /// ## Safety invariants:
        /// 1. This read token was created from [`Self::ledger`]'s cookie jar, or cloned or downgrade from one
        cookie: ManuallyDrop<<L::Cookies as CookieJar>::ReadToken<'a>>,
        generation: NonZeroU64,
        /// ## Safety invariants:
        /// 1. Convertible to reference
        ledger: NonNull<L>,
        /// ## Safety invariants:
        /// 1. Converitble to reference
        data: NonNull<ManuallyDrop<T>>,
        _phantom: PhantomData<&'a T>,
    }

    impl<'a, T, L: Ledger> ReadRalc<'a, T, L> {
        /// # Safety
        ///
        /// 1. `ledger` must be convertible to a reference
        /// 2. `cookie` must have been created from the cookie jar, or cloned or downgrade from one
        /// 3. `data` must be convertible to a reference
        pub(crate) unsafe fn from_parts(
            cookie: <L::Cookies as CookieJar>::ReadToken<'a>,
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

    impl<'a, T, L: Ledger> Clone for ReadRalc<'a, T, L> {
        fn clone(&self) -> Self {
            Self {
                // SAFETY:
                // 1. Upheld by invariant
                cookie: self.cookie.clone(),
                generation: self.generation.clone(),
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

pub use _limit_visibility::ReadRalc;

impl<'a, T, L: Ledger> ReadRalc<'a, T, L> {
    pub fn try_into_write(mut self) -> Result<WriteRalc<'a, T, L>, Self> {
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
    unsafe fn write_unsafe(&mut self) -> Result<WriteRalc<'a, T, L>, Self> {
        let generation = self.generation();
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
                WriteRalc::from_parts(cookie, generation, ledger, data)
            }),
            Err(cookie) => Err(unsafe {
                // SAFETY:
                // 1. Guaranteed by invariant on self
                // 2. Guaranteed by invariant on self
                // 3. Guaranteed by invariant on self
                Self::from_parts(cookie, generation, ledger, data)
            }),
        }
    }
}

impl<'a, T, L: Ledger> Deref for ReadRalc<'a, T, L> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // 1. Guaranteed by invariant on self
        unsafe { self.data().as_ref() }
    }
}

impl<'a, T, L: Ledger> Drop for ReadRalc<'a, T, L> {
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
