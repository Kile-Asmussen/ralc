#[cfg(test)]
use assert_impl::assert_impl;
use std::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    num::NonZeroU32,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::atomic::AtomicU64,
};

use crate::marker::parklock::ParkLock;

struct SyncLedger {
    count: AtomicU64,
    lock: ParkLock,
}

#[test]
fn syncledger_is_sync() {
    assert_impl!(Sync: SyncLedger);
}

struct SiloedLedger {
    count: u64,
    lock: Cell<u32>,
    _marker: Marker,
}

#[repr(u32)]
enum Marker {
    Marker = 1,
}

enum ThreadLocalLedger {
    Sync(&'static SyncLedger),
    Siloed(SiloedLedger),
}
