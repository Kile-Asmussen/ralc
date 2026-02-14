use std::{cell::RefCell, mem::ManuallyDrop, ops::DerefMut, ptr::NonNull};

use crate::{
    OwnedRalc, Ralc,
    allocators::{AllocatedLedger, LedgerAllocator},
    ledgerbooks::RetainingBook,
    ledgers::silo::SiloedLedger,
};

thread_local! {
    static RALC: RefCell<RetainingBook<SiloedLedger>> = RefCell::new(RetainingBook::new());
}

pub struct ThreadLocalAllocator;

impl LedgerAllocator for ThreadLocalAllocator {
    type WrappedLedger = SiloedLedger;
    type Allocator = RetainingBook<SiloedLedger>;

    const LIFETIME_NAME: &'static str = "'task";

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X {
        RALC.with(|ralc| scope(ralc.borrow_mut().deref_mut()))
    }
}

pub type ThreadLocal = AllocatedLedger<ThreadLocalAllocator>;

impl<T> OwnedRalc<T, ThreadLocal> {
    pub fn new_thread_local(data: T) -> Self {
        let data = Box::new(ManuallyDrop::new(data));
        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Self-evident
            Self::from_raw_parts(
                ThreadLocalAllocator::alloc(),
                // SAFETY:
                // 1. Guaranteed by Box
                NonNull::new_unchecked(Box::into_raw(data)),
            )
        }
    }
}

impl<T> Ralc<T, ThreadLocal> {
    pub fn new_thread_local(data: T) -> Self {
        Self::Owned(OwnedRalc::new_thread_local(data))
    }
}
