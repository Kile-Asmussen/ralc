use std::{fmt, mem::ManuallyDrop, num::NonZeroU64, ptr::NonNull};

use crate::{BorrowRalc, cookie::CookieJar, ledgers::Ledger, read::ReadRalc, write::WriteRalc};

mod _limit_visibility {

    use super::*;

    pub struct OwnedRalc<T, L: Ledger> {
        /// # Safety guarantees:
        /// 1. Convertible to a reference
        ledger: NonNull<L>,
        /// # Safety guarantees:
        /// 1. Was greated from a Box
        data: NonNull<ManuallyDrop<T>>,
    }

    impl<T, L: Ledger> OwnedRalc<T, L> {
        /// # Safety requirements
        ///
        /// 1. `ledger` must be convertible to a reference
        /// 2. `data` must be have been created from a box
        pub(crate) unsafe fn from_parts(
            ledger: NonNull<L>,
            data: NonNull<ManuallyDrop<T>>,
        ) -> Self {
            Self { ledger, data }
        }

        pub(crate) fn ledger(&self) -> &L {
            unsafe { self.ledger.as_ref() }
        }

        /// # Safety guarantees
        /// 1. The pointer returned is exactly the same as `.ledger()`
        pub(crate) fn ledger_ptr(&self) -> NonNull<L> {
            self.ledger
        }

        /// # Safety guarantees
        /// 1. The pointer returned is convertible to a reference and was
        ///    created from a Box
        pub(crate) fn data(&self) -> NonNull<ManuallyDrop<T>> {
            self.data
        }
    }
}

pub use _limit_visibility::OwnedRalc;

impl<T, L: Ledger> Drop for OwnedRalc<T, L> {
    fn drop(&mut self) {
        let generation = self.ledger().generation();
        unsafe {
            // SAFETY:
            // 1. Directly guaranteed
            // 2. This is `drop` and none of the other Ralc structs call this function in their `drop`
            L::bump(self.ledger_ptr());
        };
        std::mem::drop(unsafe {
            // SAFETY:
            // 1. Guaranteed by invariant on `Ledger::bump()`
            self.try_write_with_generation(generation)
        });
    }
}

impl<T, L: Ledger> OwnedRalc<T, L> {
    pub fn try_into_inner(self) -> Result<T, Self> {
        let res = if let Some(w) = self.try_write() {
            unsafe {
                // SAFETY:
                // 1. We forget self just below
                w.unsafe_into_inner()
            }
        } else {
            return Err(self);
        };
        unsafe {
            // SAFETY:
            // 1. Forget is called just below
            L::bump(self.ledger_ptr());
        }
        std::mem::forget(self);
        return Ok(res);
    }

    pub fn borrow(&self) -> BorrowRalc<T, L> {
        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Guaranteed directly
            BorrowRalc::from_parts(self.ledger_ptr(), self.data())
        }
    }

    pub fn read(&self) -> ReadRalc<'_, T, L> {
        let cookie = self.ledger().cookie().read();
        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Created just above
            // 3. Guarnteed directly
            ReadRalc::from_parts(
                cookie,
                self.ledger().generation(),
                self.ledger_ptr(),
                self.data(),
            )
        }
    }

    pub fn try_read(&self) -> Option<ReadRalc<'_, T, L>> {
        if let Some(cookie) = self.ledger().cookie().try_read() {
            Some(unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                // 2. Created just above
                // 3. Guarnteed directly
                ReadRalc::from_parts(
                    cookie,
                    self.ledger().generation(),
                    self.ledger_ptr(),
                    self.data(),
                )
            })
        } else {
            None
        }
    }

    pub fn write(&self) -> WriteRalc<'_, T, L> {
        let cookie = self.ledger().cookie().write();
        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Created just above
            // 3. Guaranteed directly
            WriteRalc::from_parts(
                cookie,
                self.ledger().generation(),
                self.ledger_ptr(),
                self.data(),
            )
        }
    }

    pub fn try_write(&self) -> Option<WriteRalc<'_, T, L>> {
        // SAFETY:
        // 1. Self evident
        unsafe { self.try_write_with_generation(self.ledger().generation()) }
    }

    /// # Safety requirements
    /// 1. `generation` is not greater than current value of `.ledger().generation()`
    pub unsafe fn try_write_with_generation(
        &self,
        generation: NonZeroU64,
    ) -> Option<WriteRalc<'_, T, L>> {
        if let Some(cookie) = self.ledger().cookie().try_write() {
            Some(unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                // 2. Created just above
                // 3. Guaranteed directly
                WriteRalc::from_parts(cookie, generation, self.ledger_ptr(), self.data())
            })
        } else {
            None
        }
    }
}

impl<T, L: Ledger> fmt::Debug for OwnedRalc<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedRalc").finish()
    }
}
