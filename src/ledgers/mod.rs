#[cfg(test)]
use assert_impl::assert_impl;
use std::{
    cell::Cell,
    num::NonZeroU64,
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{cookie::CookieJar, cookie::parklock::ParkLock};

mod allocator;
mod global;
mod local;

pub struct SyncLedger {
    count: AtomicU64,
    lock: ParkLock,
}

#[test]
fn syncledger_is_sync() {
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

    fn generation(&self) -> NonZeroU64 {
        unsafe { NonZeroU64::new_unchecked(self.count.load(Ordering::Relaxed)) }
    }

    // SAFETY:
    // 1. fetch_add() ensures the generation count increases
    unsafe fn bump(this: NonNull<Self>) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            // 2. Guaranteed by caller
            let this = this.as_ref();
            this.count.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[derive(Debug)]
pub struct SiloedLedger {
    count: Cell<NonZeroU64>,
    lock: Cell<u32>,
    _marker: Marker,
}

impl Default for SiloedLedger {
    fn default() -> Self {
        Self {
            count: Cell::new(unsafe { NonZeroU64::new_unchecked(1) }),
            lock: Default::default(),
            _marker: Default::default(),
        }
    }
}

#[derive(Default, Debug)]
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

    fn generation(&self) -> NonZeroU64 {
        self.count.get()
    }

    // SAFETY:
    // 1. Saturating add is used
    unsafe fn bump(this: NonNull<Self>) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller
            let this = this.as_ref();
            let n = this.generation().get().saturating_add(1);
            // SAFETY:
            // Not possible for above calculation to result in zero
            let n = NonZeroU64::new_unchecked(n);
            this.count.set(n);
        };
    }
}

pub trait Ledger {
    type Cookies: CookieJar;
    fn cookie(&self) -> &Self::Cookies;
    fn generation(&self) -> NonZeroU64;

    /// # Safety requirements
    /// 1. `this` is convertible to a reference
    /// 2. This function is only called once on this pointer
    ///
    /// # Safety guarantees
    /// 1. The value of `.generation()` will either be increased or u64::MAX after this function returns
    unsafe fn bump(this: NonNull<Self>);
}
