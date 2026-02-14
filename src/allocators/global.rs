use parking_lot::Mutex;

use crate::{
    OwnedRalc,
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

pub type Global = AllocatedLedger<GlobalAllocator>;

impl<T> OwnedRalc<T, Global> {
    #[cfg(test)]
    pub fn global_ledger(&self) -> &'static Global {
        unsafe {
            // SAFETY:
            // 1. Global ledgers are by nature, static
            self.ledger_ptr().as_ref()
        }
    }
}

#[test]
fn send_sync() {
    use crate::OwnedRalc;
    use assert_impl::assert_impl;
    assert_impl!(Send: OwnedRalc<i32, Global>);
}
