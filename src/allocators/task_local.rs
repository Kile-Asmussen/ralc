#[cfg(feature = "bumpalo")]
use std::ptr::NonNull;
use std::{cell::RefCell, ops::DerefMut};

use crate::{
    allocators::{AllocatedLedger, LedgerAllocator},
    ledgers::silo::SiloedLedger,
};

#[cfg(not(feature = "bumpalo"))]
type Book<L> = crate::ledgerbooks::RetainingBook<L>;
#[cfg(feature = "bumpalo")]
type Book<L> = crate::ledgerbooks::BumpyBook<NonNull<L>, L>;

tokio::task_local! {
    static RALC: RefCell<Book<SiloedLedger>>;
}

pub struct TaskLocalAllocator;

#[allow(dead_code)]
pub trait FutureExt: Future + Sized {
    async fn with_ralcs(self) -> Self::Output {
        RALC.sync_scope(RefCell::new(Book::new()), move || self)
            .await
    }
}

impl<F: Future + Sized> FutureExt for F {}

impl LedgerAllocator for TaskLocalAllocator {
    type WrappedLedger = SiloedLedger;
    type Allocator = Book<SiloedLedger>;

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X {
        RALC.with(|rc| scope(rc.borrow_mut().deref_mut()))
    }
}

pub type TaskLocal = AllocatedLedger<TaskLocalAllocator>;
