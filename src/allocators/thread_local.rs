use std::{cell::RefCell, ops::DerefMut};

use crate::{
    allocators::{AllocatedLedger, LedgerAllocator},
    ledgers::silo::SiloedLedger,
};

#[cfg(not(feature = "bumpalo"))]
type Book<L> = crate::ledgerbooks::RetainingBook<L>;
#[cfg(feature = "bumpalo")]
type Book<L> = crate::ledgerbooks::BumpyBook<std::ptr::NonNull<L>, L>;

thread_local! {
    static RALC: RefCell<Book<SiloedLedger>> = RefCell::new(Book::new());
}

pub struct ThreadLocalAllocator;

impl LedgerAllocator for ThreadLocalAllocator {
    type WrappedLedger = SiloedLedger;
    type Allocator = Book<SiloedLedger>;

    const LIFETIME_NAME: &'static str = "'thread";

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X {
        RALC.with(|ralc| scope(ralc.borrow_mut().deref_mut()))
    }
}

pub type ThreadLocal = AllocatedLedger<ThreadLocalAllocator>;
