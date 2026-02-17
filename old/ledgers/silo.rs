use std::{cell::Cell, num::NonZeroU64};

use crate::{cookie::CookieJar, ledgers::Ledger};

#[derive(Debug)]
pub struct SiloedLedger {
    count: Cell<NonZeroU64>,
    lock: Cell<u32>,
    _marker: Marker,
}

impl Default for SiloedLedger {
    #[inline]
    fn default() -> Self {
        Self {
            count: Cell::new(unsafe { NonZeroU64::new_unchecked(1) }),
            lock: Cell::INIT,
            _marker: Marker::Marker,
        }
    }
}

impl Clone for SiloedLedger {
    #[inline]
    fn clone(&self) -> Self {
        Default::default()
    }
}

#[test]
fn send_sync() {
    use assert_impl::assert_impl;
    assert_impl!(!Sync: SiloedLedger);
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(u32)]
enum Marker {
    #[default]
    Marker = 1,
}

impl Ledger for SiloedLedger {
    type Cookies = Cell<u32>;

    fn cookie(&self) -> &Self::Cookies {
        &self.lock
    }

    fn reallocation(&self) -> NonZeroU64 {
        self.count.get()
    }

    // SAFETY:
    // 1. `u64::saturating_add` is used.
    unsafe fn bump_impl(&self) {
        let n = self.reallocation().get().saturating_add(1);
        let n = unsafe {
            // SAFETY:
            // Not possible for above calculation to result in zero
            NonZeroU64::new_unchecked(n)
        };
        self.count.set(n);
    }

    fn read_failure() -> ! {
        std::panic!("thread-local cell is already borrowed")
    }

    fn write_failure() -> ! {
        std::panic!("thread-local cell is already borrowed")
    }
}
