use std::ptr::NonNull;

use crate::ledgers::Ledger;

pub trait LedgerAllocator {
    type WrappedLedger: Ledger + Default;

    /// # Safety guarantees
    /// 1. The returned pointer is convertible to a reference that is valid for as long
    ///    as the allocator exists (thread local, static, etc.)
    /// 2. The ledger's `.generation()` is less than `NonZeroU64::MAX`
    fn alloc() -> NonNull<AllocatedLedger<Self>>;

    /// # Safety requirements
    /// 1. `ledger` is a pointer returned by `alloc`
    /// 2. `free` is not called for the same pointer twice
    unsafe fn free(ledger: NonNull<AllocatedLedger<Self>>);
}

pub use _limit_visibility::AllocatedLedger;

mod _limit_visibility {
    use super::*;

    #[repr(transparent)]
    /// # Safety invariants
    /// 1. This object is only obtained as a `NonNull` returned from `A::alloc()`
    pub struct AllocatedLedger<A: LedgerAllocator + ?Sized>(<A as LedgerAllocator>::WrappedLedger);

    impl<A: LedgerAllocator> AllocatedLedger<A> {
        /// # Safety requirements
        /// 1. This is only called for the purposes of providing a `NonNull` pointer
        ///    to be returned from `LedgerAllocator::alloc`
        pub(crate) unsafe fn new() -> Self {
            Self(Default::default())
        }

        pub(crate) fn inner(&self) -> &<A as LedgerAllocator>::WrappedLedger {
            &self.0
        }
    }
}

impl<A: LedgerAllocator> AllocatedLedger<A> {
    fn underlying(this: NonNull<Self>) -> NonNull<A::WrappedLedger> {
        // SAFETY:
        // 1. Guaranteed by `repr(transparent)`
        unsafe { std::mem::transmute(this) }
    }
}

impl<A: LedgerAllocator> Ledger for AllocatedLedger<A> {
    type Cookies = <A::WrappedLedger as Ledger>::Cookies;

    fn cookie(&self) -> &Self::Cookies {
        self.inner().cookie()
    }

    fn generation(&self) -> std::num::NonZeroU64 {
        self.inner().generation()
    }

    unsafe fn bump(this: NonNull<Self>) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            // 2. Guaranteed by caller
            A::WrappedLedger::bump(Self::underlying(this));
            // SAFETY:
            // 1. Guaranteed by invariant
            // 2. Guaranteed by caller
            A::free(this);
        }
    }
}
