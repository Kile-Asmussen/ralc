use std::ptr::NonNull;

use crate::{ledgerbooks::LedgerBook, ledgers::Ledger};

mod global;
pub use global::{GlobalAllocator, GlobalLedger};
mod pool;
pub use pool::{PoolAllocator, PoolLedger};
#[cfg(feature = "tokio")]
mod task_local;
#[cfg(feature = "tokio")]
pub use task_local::{TaskLocalAllocator, TaskLocalLedger};
mod thread_local;
pub use thread_local::{ThreadLocalAllocator, ThreadLocalLedger};

pub trait LedgerAllocator {
    type WrappedLedger: Ledger + Default;
    type Allocator: LedgerBook<Self::WrappedLedger>;

    const LIFETIME_NAME: &'static str = "'_";

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X;

    fn alloc() -> NonNull<AllocatedLedger<Self>> {
        Self::with(|a| unsafe {
            // SAFETY:
            // 1. Self-evident
            AllocatedLedger::from_inner_ptr(a.allocate(Default::default))
        })
    }

    /// # Safety requirements
    /// 1. `ledger` is a pointer returned by `alloc`
    /// 2. `free` is not called for the same pointer twice
    unsafe fn free(ledger: NonNull<AllocatedLedger<Self>>) {
        Self::with(|a| unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Guaranteed by caller
            a.deallocate(AllocatedLedger::into_inner_ptr(ledger))
        })
    }
}

pub use _limit_visibility::AllocatedLedger;

mod _limit_visibility {
    use super::*;

    /// # Safety invariants
    /// 1. This object is only obtained as a `NonNull` returned from `A::alloc()`
    #[repr(transparent)]
    pub struct AllocatedLedger<A: LedgerAllocator + ?Sized> {
        inner: A::WrappedLedger,
    }

    impl<A: LedgerAllocator + ?Sized> AllocatedLedger<A> {
        pub(crate) fn inner(&self) -> &A::WrappedLedger {
            &self.inner
        }

        /// # Safety requirements
        /// 1. `ptr` must have been allocated with the allocator's ledgerbook
        pub(crate) unsafe fn from_inner_ptr(ptr: NonNull<A::WrappedLedger>) -> NonNull<Self> {
            ptr.cast()
        }

        /// # Safety guarantees
        /// 1. The resulting pointer originates from the allocator's ledgerbook
        pub(crate) fn into_inner_ptr(ptr: NonNull<Self>) -> NonNull<A::WrappedLedger> {
            ptr.cast()
        }
    }
}

impl<A: LedgerAllocator + ?Sized> Ledger for AllocatedLedger<A> {
    type Cookies = <<A as LedgerAllocator>::WrappedLedger as Ledger>::Cookies;

    fn cookie(&self) -> &Self::Cookies {
        self.inner().cookie()
    }

    fn reallocation(&self) -> std::num::NonZeroU64 {
        self.inner().reallocation()
    }

    // SAFETY:
    // 1. Delegated
    unsafe fn bump_impl(&self) {
        unsafe { self.inner().bump_impl() };
    }

    unsafe fn free(this: NonNull<Self>) {
        A::with(|a| unsafe {
            // SAFETY:
            // 1. Guaranteed by invariant
            // 2. Guaranteed by caller
            a.deallocate(Self::into_inner_ptr(this));
        });
    }

    const LIFETIME_NAME: &'static str = A::LIFETIME_NAME;
}
