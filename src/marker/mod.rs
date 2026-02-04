#[cfg(test)]
use assert_impl::assert_impl;
use std::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    num::NonZeroU32,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::atomic::AtomicU32,
};

mod celllock;
mod cookie;

#[cfg(feature = "parking_lot")]
mod parklock;
#[cfg(feature = "parking_lot")]
use parklock::ParkLock as Lock;

#[cfg(not(feature = "parking_lot"))]
mod stdlock;
#[cfg(not(feature = "parking_lot"))]
use stdlock::StdLock as Lock;

/// Global, atomic Reallocation Count
struct Gralc {
    count: AtomicU32,
    lock: Lock,
}

#[test]
fn aralc_is_sync() {
    assert_impl!(Sync: Gralc);
}

/// Local Atomic Reallocation Count
struct Lralc {
    lock: Cell<u32>,
    count: u64,
}

#[test]
fn lralc_is_sync() {
    assert_impl!(!Sync: Lralc);
}

#[derive(Clone)]
struct ReadToken<'a>(&'a Lralc);

impl<'a> Drop for ReadToken<'a> {
    fn drop(&mut self) {
        self.0.lock.update(|n| n - 1);
    }
}

struct WriteToken<'a>(&'a Lralc);
