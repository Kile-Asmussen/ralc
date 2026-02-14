use std::{
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

use tokio::sync::Semaphore;

use crate::ledgers::Ledger;

#[derive(Debug)]
#[allow(dead_code)]
pub struct SemLedger {
    count: AtomicU64,
    lock: Semaphore,
}

impl Default for SemLedger {
    fn default() -> Self {
        SemLedger {
            count: AtomicU64::new(1),
            lock: Semaphore::new(u32::MAX as usize),
        }
    }
}

impl Ledger for SemLedger {
    type Cookies = Semaphore;

    fn cookie(&self) -> &Self::Cookies {
        &self.lock
    }

    fn reallocation(&self) -> NonZeroU64 {
        NonZeroU64::new(self.count.load(Ordering::Relaxed)).unwrap()
    }

    // SAFETY:
    // 1. AtomicU64::fetch_add is used
    unsafe fn bump_impl(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}
