use std::{cell::RefCell, ops::DerefMut};

use crate::{
    allocators::{AllocatedLedger, LedgerAllocator},
    ledgerbooks::RetainingBook,
    ledgers::silo::SiloedLedger,
};

tokio::task_local! {
    static RALC: RefCell<RetainingBook<SiloedLedger>>;
}

pub struct TaskLocalAllocator;

#[allow(dead_code)]
pub trait FutureExt: Future + Sized {
    async fn with_ralcs(self) -> Self::Output {
        RALC.sync_scope(RefCell::new(RetainingBook::new()), move || self)
            .await
    }
}

impl LedgerAllocator for TaskLocalAllocator {
    type WrappedLedger = SiloedLedger;
    type Allocator = RetainingBook<SiloedLedger>;

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X {
        RALC.with(|rc| scope(rc.borrow_mut().deref_mut()))
    }
}

pub type TaskLocal = AllocatedLedger<TaskLocalAllocator>;
