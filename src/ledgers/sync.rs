use std::{
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{cookie::parking_lot::ParkLock, ledgers::Ledger};

pub struct SyncLedger {
    count: AtomicU64,
    lock: ParkLock,
}

#[test]
fn syncledger_is_sync() {
    use assert_impl::assert_impl;

    assert_impl!(Sync: SyncLedger);
}

impl Default for SyncLedger {
    fn default() -> Self {
        Self {
            count: AtomicU64::new(1),
            lock: ParkLock::new(),
        }
    }
}

impl SyncLedger {
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(1),
            lock: ParkLock::new(),
        }
    }
}

impl Ledger for SyncLedger {
    type Cookies = ParkLock;

    fn cookie(&self) -> &Self::Cookies {
        &self.lock
    }

    fn reallocation(&self) -> NonZeroU64 {
        NonZeroU64::new(self.count.load(Ordering::Relaxed)).unwrap()
    }

    // SAFETY:
    // 1. AtomicU64::fetch_add is used.
    unsafe fn bump_impl(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}
