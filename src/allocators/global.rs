use std::{mem::ManuallyDrop, ptr::NonNull};

use parking_lot::Mutex;

use crate::{
    OwnedRalc, Ralc,
    allocators::{AllocatedLedger, LedgerAllocator},
    ledgerbooks::LeakyBook,
    ledgers::sync::SyncLedger,
};

pub struct GlobalAllocator;

impl LedgerAllocator for GlobalAllocator {
    type WrappedLedger = SyncLedger;
    type Allocator = LeakyBook<SyncLedger>;

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X {
        static RALC: Mutex<LeakyBook<SyncLedger>> = Mutex::new(LeakyBook::new());
        scope(&mut RALC.lock())
    }

    const LIFETIME_NAME: &'static str = "'static";
}

pub type GlobalLedger = AllocatedLedger<GlobalAllocator>;

impl<T: Send + Sync> OwnedRalc<T, GlobalLedger> {
    pub fn new_global(data: T) -> Self {
        let data = Box::new(ManuallyDrop::new(data));
        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Self-evident
            Self::from_parts(
                GlobalAllocator::alloc(),
                // SAFETY:
                // 1. Guaranteed by Box
                NonNull::new_unchecked(Box::into_raw(data)),
            )
        }
    }

    pub fn global_ledger(&self) -> &'static GlobalLedger {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by GlobalAllocator's data existing for the static lifetime
            self.ledger_ptr().as_ref()
        }
    }
}

impl<T: Send + Sync> Ralc<T, GlobalLedger> {
    pub fn new_global(data: T) -> Self {
        Self::Owned(OwnedRalc::new_global(data))
    }
}

#[test]
fn send_sync() {
    use assert_impl::assert_impl;
    assert_impl!(Send: OwnedRalc<i32, GlobalLedger>);
}
