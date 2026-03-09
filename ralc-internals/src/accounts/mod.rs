#![allow(unused)]
use std::{cell::Cell, num::NonZeroU64, ops::Deref, ptr::NonNull, sync::atomic::AtomicU64};

use crate::accounts::{balances::Balance, freeable::Freeable, permits::Permits};

pub mod balances;
pub mod freeable;
pub mod permits;

pub trait Account: Freeable + Balance {}

pub use private::AccPtr;

// https://github.com/rust-lang/rust-project-goals/issues/273
mod private {
    use crate::accounts::Account;
    use std::{ops::Deref, ptr::NonNull};

    /// A pointer to an account allocated to guard and track a single allocation.
    #[repr(transparent)]
    pub struct AccPtr<A: Account + ?Sized> {
        /// # Safety invariant
        /// 1. ONE of these hold:
        ///    1. This is a `'static` reference.
        ///    2. This is a reference to a thread-local and this type is not `Send`
        ///    3. This is a reference to a shorter lifetime contained in the type `A`.
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
