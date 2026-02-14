use std::{cell::RefCell, mem::ManuallyDrop, ops::DerefMut, ptr::NonNull};

use crate::{
    OwnedRalc, Ralc,
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
    async fn ralc(self) -> Self::Output {
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

pub type TaskLocalLedger = AllocatedLedger<TaskLocalAllocator>;

impl<T> OwnedRalc<T, TaskLocalLedger> {
    pub fn new_task_local(data: T) -> Self {
        let data = Box::new(ManuallyDrop::new(data));
        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Self-evident
            Self::from_parts(
                TaskLocalAllocator::alloc(),
                // SAFETY:
                // 1. Guaranteed by Box
                NonNull::new_unchecked(Box::into_raw(data)),
            )
        }
    }
}

impl<T> Ralc<T, TaskLocalLedger> {
    pub fn new_task_local(data: T) -> Self {
        Self::Owned(OwnedRalc::new_task_local(data))
    }
}
