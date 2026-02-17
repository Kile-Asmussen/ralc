use std::{
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    cookie::{CookieJar, parking_lot::ParkLock},
    ledgers::Ledger,
};

pub struct SyncLedger {
    count: AtomicU64,
    lock: ParkLock,
}

impl Clone for SyncLedger {
    #[inline]
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Default for SyncLedger {
    #[inline]
    fn default() -> Self {
        Self {
            count: AtomicU64::new(1),
            lock: ParkLock::INIT,
        }
    }
}

#[test]
fn send_sync() {
    use assert_impl::assert_impl;
    assert_impl!(Sync: SyncLedger);
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
