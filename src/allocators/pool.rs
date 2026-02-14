use std::{cell::UnsafeCell, marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use crate::{
    OwnedRalc,
    allocators::pool::_limit_visibility::PoolAllocatedLedger,
    cookie::CookieJar,
    ledgerbooks::{LedgerBook, RetainingBook},
    ledgers::Ledger,
};

pub struct PoolAllocator<L: Ledger + Default + 'static> {
    cookie: L::Cookies,
    book: UnsafeCell<RetainingBook<PoolAllocatedLedger<L>>>,
}

unsafe impl<C: Sync, L: Ledger<Cookies = C> + Default> Sync for PoolAllocator<L> {}

impl<L: Ledger + Default> PoolAllocator<L> {
    pub fn new() -> Self {
        Self {
            cookie: L::Cookies::default(),
            book: UnsafeCell::new(RetainingBook::new()),
        }
    }

    pub fn ralc<'a, T>(&'a self, value: T) -> OwnedRalc<T, PoolLedger<'a, L>> {
        unsafe {
            // SAFETY:
            // 1. Box is never null
            let data = NonNull::new_unchecked(Box::into_raw(Box::new(ManuallyDrop::new(value))));

            OwnedRalc::from_parts(self.alloc_ledger(), data)
        }
    }

    fn alloc_ledger<'a>(&'a self) -> NonNull<PoolLedger<'a, L>> {
        let _cookie = self.cookie.write().unwrap_or_else(|| L::write_failure());
        let ledger = unsafe {
            // SAFETY:
            // 1. The cookie grants permission to access
            let book = self.book.get().as_mut().unwrap();
            book.allocate(|| PoolLedger::new(self).downcast())
        };
        ledger.cast()
    }

    /// # Safety requirements
    /// 1. ledger is a return from `alloc_ledger`
    /// 2. free is not called for the same pointer twice
    unsafe fn dealloc_ledger(&self, ledger: NonNull<PoolLedger<'_, L>>) {
        let _cookie = self.cookie.write().unwrap_or_else(|| L::write_failure());
        unsafe {
            // SAFETY:
            // 1. The cookie grants permission to access
            let book = self.book.get().as_mut().unwrap();

            // SAFETY:
            // 1. The cookie grants permission to access
            book.deallocate(ledger.cast());
        }
    }
}

pub use _limit_visibility::PoolLedger;

mod _limit_visibility {
    use super::*;

    pub(super) struct PoolAllocatedLedger<L: Ledger + Default + 'static> {
        /// # Safety invariant
        /// 1. This pointer is valid for shared references
        pool: NonNull<PoolAllocator<L>>,
        inner: L,
    }

    unsafe impl<L: Ledger + Default + 'static + Sync> Sync for PoolAllocatedLedger<L> {}

    impl<L: Ledger + Default + 'static> PoolAllocatedLedger<L> {
        pub(super) unsafe fn inner(&self) -> &L {
            &self.inner
        }
    }

    #[repr(transparent)]
    pub struct PoolLedger<'a, L: Ledger + Default + 'static> {
        /// # Safety invariant
        /// 1. The inner pointer of this type is valid for shared references
        ///    with lifetime of at least `'a`
        raw: PoolAllocatedLedger<L>,
        _phantom: PhantomData<&'a PoolAllocator<L>>,
    }

    impl<'a, L: Ledger + Default> PoolLedger<'a, L> {
        pub(super) fn new(pool: &'a PoolAllocator<L>) -> Self {
            Self {
                raw: PoolAllocatedLedger {
                    pool: NonNull::from_ref(pool),
                    inner: L::default(),
                },
                _phantom: PhantomData,
            }
        }

        pub(super) fn downcast(self) -> PoolAllocatedLedger<L> {
            self.raw
        }

        pub(super) fn raw(&self) -> &PoolAllocatedLedger<L> {
            &self.raw
        }

        pub fn pool(&self) -> &'a PoolAllocator<L> {
            unsafe {
                // SAFETY:
                // 1. Guaranteed by invariant
                self.raw.pool.as_ref()
            }
        }

        pub(crate) fn inner(&self) -> &L {
            &self.raw.inner
        }
    }
}

impl<L: Ledger + Default> Ledger for PoolAllocatedLedger<L> {
    type Cookies = L::Cookies;

    fn cookie(&self) -> &Self::Cookies {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by invariant
            self.inner()
        }
        .cookie()
    }

    fn reallocation(&self) -> std::num::NonZeroU64 {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by invariant
            self.inner()
        }
        .reallocation()
    }

    // SAFETY:
    // 1. Delegated
    unsafe fn bump_impl(&self) {
        unsafe {
            self.inner().bump_impl();
        }
    }

    unsafe fn free(_this: NonNull<Self>) {}
}

impl<'a, L: Ledger + Default> Ledger for PoolLedger<'a, L> {
    type Cookies = <L as Ledger>::Cookies;

    fn cookie(&self) -> &Self::Cookies {
        self.inner().cookie()
    }

    fn reallocation(&self) -> std::num::NonZeroU64 {
        self.inner().reallocation()
    }

    // SAFETY:
    // 1. Delegated
    unsafe fn bump_impl(&self) {
        unsafe {
            self.raw().bump_impl();
        }
    }

    const LIFETIME_NAME: &'static str = "'a";

    unsafe fn free(this: NonNull<Self>) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            this.as_ref()
                .pool()
                // SAFETY:
                // 1. Guaranteed by caller
                // 2. Guaranteed by caller
                .dealloc_ledger(this)
        };
    }

    fn read_failure() -> ! {
        L::read_failure()
    }

    fn write_failure() -> ! {
        L::write_failure()
    }
}
