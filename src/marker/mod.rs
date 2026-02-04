#[cfg(test)]
use assert_impl::assert_impl;
use std::sync::atomic::AtomicU32;

#[cfg(feature = "parking_lot")]
mod parklock;
#[cfg(feature = "parking_lot")]
use parklock::*;

#[cfg(not(feature = "parking_lot"))]
mod stdlock;
#[cfg(not(feature = "parking_lot"))]
use stdlock::*;

/// Send/Sync Atomic Reallocation Count
struct Aralc {
    count: AtomicU32,
    lock: Lock,
}

#[test]
fn aralc_is_sync() {
    assert_impl!(Sync: Aralc)
}

/// Local Atomic Reallocation Count
struct Larc {
    lock: i32,
    count: u32,
}

/// Thread-Local
struct Uarc {}
