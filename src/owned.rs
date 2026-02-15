use std::{
    fmt::{self},
    mem::ManuallyDrop,
    num::NonZeroU64,
    ptr::NonNull,
};

use crate::{
    BorrowRalc, NoAccess, Result,
    cookie::CookieJar,
    ledgers::{Ledger, LedgerExt},
    mut_::RalcMut,
    ref_::RalcRef,
};

mod _limit_visibility {

    use super::*;

    pub struct OwnedRalc<T, L: Ledger> {
        /// # Safety guarantees:
        /// 1. Convertible to a reference
        /// 2. Referenced ledger has a [`.reallocation()`](Ledger::reallocation) count that is not NonZeroU64::MAX
        ledger: NonNull<L>,
        /// # Safety guarantees:
        /// 1. Was greated from a Box
        data: NonNull<ManuallyDrop<T>>,
    }

    unsafe impl<T: Send + Sync, L: Ledger + Sync> Send for OwnedRalc<T, L> {}
    unsafe impl<T: Sync, L: Ledger + Sync> Sync for OwnedRalc<T, L> {}

    impl<T, L: Ledger> OwnedRalc<T, L> {
        /// # Safety requirements
        ///
        /// 1. `ledger` must be convertible to a reference
        /// 2. `data` must be have been created from a box
        /// 3. ledger has a [`.reallocation()`](Ledger::reallocation) count that is not `NonZeroU64::MAX`
        pub(crate) unsafe fn from_raw_parts(
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
        let reallocation = self.ledger().reallocation();
        self.ledger().bump();
        std::mem::drop(unsafe {
            // SAFETY:
            // 1. Guaranteed by `LedgerExt:bump()`
            self.try_write_with_reallocation(reallocation)
        });
    }
}

impl<T, L: Ledger> OwnedRalc<T, L> {
    pub fn try_into_box(self) -> std::result::Result<Box<T>, Self> {
        let res = if let Ok(w) = self.try_write() {
            unsafe {
                // SAFETY:
                // 1. We forget self just below
                w.unsafe_take_box()
            }
        } else {
            return Err(self);
        };
        self.ledger().bump();
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

    pub fn read(&self) -> Result<RalcRef<'_, T, L>> {
        let Some(cookie) = self.ledger().cookie().read() else {
            return Err(NoAccess::Deadlock);
        };

        Ok(unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Created just above
            // 3. Guarnteed directly
            RalcRef::from_parts(
                cookie,
                self.ledger().reallocation(),
                self.ledger_ptr(),
                self.data(),
            )
        })
    }

    pub fn try_read(&self) -> Result<RalcRef<'_, T, L>> {
        if let Some(cookie) = self.ledger().cookie().try_read() {
            Ok(unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                // 2. Created just above
                // 3. Guarnteed directly
                RalcRef::from_parts(
                    cookie,
                    self.ledger().reallocation(),
                    self.ledger_ptr(),
                    self.data(),
                )
            })
        } else {
            Err(NoAccess::Blocked)
        }
    }

    pub fn write(&self) -> Result<RalcMut<'_, T, L>> {
        let Some(cookie) = self.ledger().cookie().write() else {
            return Err(NoAccess::Deadlock);
        };

        Ok(unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Created just above
            // 3. Guaranteed directly
            RalcMut::from_parts(
                cookie,
                self.ledger().reallocation(),
                self.ledger_ptr(),
                self.data(),
            )
        })
    }

    pub fn try_write(&self) -> Result<RalcMut<'_, T, L>> {
        // SAFETY:
        // 1. Self evident
        unsafe { self.try_write_with_reallocation(self.ledger().reallocation()) }
    }

    /// # Safety requirements
    /// 1. `reallocation` is not greater than current value of [`.ledger().reallocation()`](Ledger::reallocation)
    #[inline]
    pub unsafe fn try_write_with_reallocation(
        &self,
        reallocation: NonZeroU64,
    ) -> Result<RalcMut<'_, T, L>> {
        if let Some(cookie) = self.ledger().cookie().try_write() {
            Ok(unsafe {
                // SAFETY:
                // 1. Guaranteed directly
                // 2. Created just above
                // 3. Guaranteed directly
                RalcMut::from_parts(cookie, reallocation, self.ledger_ptr(), self.data())
            })
        } else {
            Err(NoAccess::Blocked)
        }
    }
}

impl<T: fmt::Debug, L: Ledger> fmt::Debug for OwnedRalc<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_tuple("OwnedRalc")
        } else {
            f.debug_tuple(&format!("box {} ralc", L::LIFETIME_NAME))
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

impl<T: fmt::Display, L: Ledger> fmt::Display for OwnedRalc<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if f.alternate() { " (owned)" } else { "" };
        if let Ok(r) = self.try_read() {
            write!(f, "{}{}", r, marker)
        } else {
            write!(f, "<unavailable>{}", marker)
        }
    }
}
