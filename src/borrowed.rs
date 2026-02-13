use std::{mem::ManuallyDrop, num::NonZeroU64, ptr::NonNull};

use crate::{ReadRalc, WriteRalc, cookie::CookieJar, ledgers::Ledger};

mod _limit_visibility {
    use super::*;

    pub struct BorrowRalc<T, L: Ledger> {
        generation: NonZeroU64,
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
                generation: unsafe {
                    // SAFETY:
                    // 1. Guaranteed by caller
                    ledger.as_ref()
                }
                .generation(),
                ledger,
                data,
            }
        }

        pub fn ledger(&self) -> &L {
            // SAFETY:
            // 1. Guaranteed by invariant
            unsafe { self.ledger.as_ref() }
        }

        pub fn generation(&self) -> NonZeroU64 {
            self.generation
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
                generation: self.generation,
                ledger: self.ledger,
                data: self.data,
            }
        }
    }

    impl<T, L: Ledger> Copy for BorrowRalc<T, L> {}
}

pub use _limit_visibility::BorrowRalc;

impl<T, L: Ledger> BorrowRalc<T, L> {
    pub fn check(&self) -> bool {
        self.ledger().generation() == self.generation()
    }

    pub fn read(&self) -> Option<ReadRalc<'_, T, L>> {
        if !self.check() {
            return None;
        }

        let cookie = self.ledger().cookie().read();

        Some(unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Cookie is generated just above
            // 3. Guaranteed directly
            ReadRalc::from_parts(cookie, self.generation(), self.ledger_ptr(), self.data())
        })
    }

    pub fn write(&self) -> Option<WriteRalc<'_, T, L>> {
        if !self.check() {
            return None;
        }

        let cookie = self.ledger().cookie().write();
        Some(unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Created just above
            // 3. Guaranteed directly
            WriteRalc::from_parts(cookie, self.generation(), self.ledger_ptr(), self.data())
        })
    }

    pub fn try_read(&self) -> Option<ReadRalc<'_, T, L>> {
        if !self.check() {
            return None;
        }

        if let Some(cookie) = self.ledger().cookie().try_read() {
            Some(unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                // 2. Cookie is generated just above
                // 3. Guaranteed directly
                ReadRalc::from_parts(cookie, self.generation(), self.ledger_ptr(), self.data())
            })
        } else {
            None
        }
    }

    pub fn try_write(&self) -> Option<WriteRalc<'_, T, L>> {
        if !self.check() {
            return None;
        }

        if let Some(cookie) = self.ledger().cookie().try_write() {
            Some(unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                // 2. Created just above
                // 3. Guaranteed directly
                WriteRalc::from_parts(cookie, self.generation(), self.ledger_ptr(), self.data())
            })
        } else {
            None
        }
    }
}
