#![allow(unused)]
use std::{cell::Cell, num::NonZeroU64, ops::Deref, ptr::NonNull, sync::atomic::AtomicU64};

use crate::accounts::{balances::Balance, permits::Permits};

pub mod balances;
pub mod init;
pub mod permits;
pub mod simple;

pub trait Account: Freeable + Balance {}

/// # Safety reuirements
/// 1. `free` must drop `mut` permit.
pub unsafe trait Freeable: Permits {
    /// # Safety requirements:
    /// 1. After calling it, no other interactions may be made with this object.
    /// 2. `mut` permit is invoked on [`.permits()`](Self::permits)
    unsafe fn free(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller.
            self.drop_mut();
        }
        // SAFETY:
        // 1. See above
    }
}

#[test]
fn test() {
    size_of::<AccPtr<dyn Account>>();
}

use parking_lot::RawRwLock;
pub(crate) use private::AccPtr;
mod private {
    use crate::accounts::Account;
    use std::{ops::Deref, ptr::NonNull};

    #[repr(transparent)]
    pub(crate) struct AccPtr<A: Account + ?Sized> {
        /// # Safety invariant:
        /// 1. ONE of these hold:
        ///    1. This is a `'static` reference.
        ///    2. This is a reference to a thread-local and this type is not `Send`
        ///    3. This is a reference to a shorter lifetime contained in `A`.
        account: NonNull<A>,
    }
    impl<A: Account> Clone for AccPtr<A> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<A: Account> Copy for AccPtr<A> {}

    unsafe impl<A: Account + Sync> Send for AccPtr<A> {}
    unsafe impl<A: Account + Sync> Sync for AccPtr<A> {}

    impl<A: Account> AccPtr<A> {
        /// # Safety requirements:
        ///
        /// 1. ONE of the following must hold:
        ///    1. `account` is a reference with `'static` lifetime.
        ///    2. `account` has the lifetime thread locals and `Self: !Send`.
        ///    3. `account` has a shorter lifetime but the type of `A` constrains it.
        pub unsafe fn new(account: &A) -> Self {
            Self {
                // SAFETY:
                // 1. Guaranteed by caller.
                account: NonNull::from_ref(account),
            }
        }
    }

    impl<A: Account> Deref for AccPtr<A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            // SAFETY:
            // 1.1 this dereferences a static.
            // 1.2 this dereferences a thread local.
            // 1.3 this dereferences in the liftetime of `A`'s type.
            unsafe { self.account.as_ref() }
        }
    }
}
