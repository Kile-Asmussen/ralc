use std::{cell::Cell, sync::atomic::AtomicU64};

use parking_lot::{RawRwLock, lock_api};

pub trait Init {
    fn init() -> Self
    where
        Self: Sized;
}

impl Init for AtomicU64 {
    fn init() -> Self
    where
        Self: Sized,
    {
        AtomicU64::new(0)
    }
}

impl Init for Cell<u64> {
    fn init() -> Self
    where
        Self: Sized,
    {
        Cell::new(0)
    }
}

impl Init for Cell<u32> {
    fn init() -> Self
    where
        Self: Sized,
    {
        Cell::new(0)
    }
}

impl Init for RawRwLock {
    fn init() -> Self
    where
        Self: Sized,
    {
        <RawRwLock as lock_api::RawRwLock>::INIT
    }
}
