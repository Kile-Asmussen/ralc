use parking_lot::Mutex;

use crate::{
    OwnedRalc,
    allocators::{AllocatedLedger, LedgerAllocator},
    ledgers::sync::SyncLedger,
};

#[cfg(not(feature = "bumpalo"))]
type Book<L> = crate::ledgerbooks::LeakyBook<L>;
#[cfg(feature = "bumpalo")]
type Book<L> = crate::ledgerbooks::BumpyBook<&'static L, L>;

pub struct GlobalAllocator;

impl LedgerAllocator for GlobalAllocator {
    type WrappedLedger = SyncLedger;
    type Allocator = Book<SyncLedger>;

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X {
        static RALC: Mutex<Book<SyncLedger>> = Mutex::new(Book::new());
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
