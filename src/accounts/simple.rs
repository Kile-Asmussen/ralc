use std::{cell::Cell, sync::atomic::AtomicU64};

use parking_lot::RawRwLock;

use crate::accounts::{
    Account, Freeable,
    balances::Balance,
    permits::{Permits, WaitPermits},
};

pub struct SimpleAccount<B: Balance, P: Permits> {
    balance: B,
    permits: P,
}

pub type SyncAccount = SimpleAccount<AtomicU64, RawRwLock>;
pub type CellAccount = SimpleAccount<Cell<u64>, Cell<u32>>;

// SAFETY:
// Default impls only.
unsafe impl<B: Balance, P: Permits> Freeable for SimpleAccount<B, P> {}

impl<B: Balance, P: Permits> Account for SimpleAccount<B, P> {}

// SAFETY:
// 1. delegated implementation.
unsafe impl<B: Balance, P: Permits> Balance for SimpleAccount<B, P> {
    const INIT: Self = Self {
        balance: B::INIT,
        permits: P::INIT,
    };

    fn invalidate(&self) {
        self.balance.invalidate();
    }

    fn check(&self) -> u64 {
        self.balance.check()
    }
}

impl<B: Balance, P: Permits> Permits for SimpleAccount<B, P> {
    const INIT: Self = Self {
        balance: B::INIT,
        permits: P::INIT,
    };

    fn try_ref(&self) -> bool {
        self.permits.try_ref()
    }

    fn try_mut(&self) -> bool {
        self.permits.try_mut()
    }

    unsafe fn try_ref_to_mut(&self) -> bool {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller.
            self.permits.try_ref_to_mut()
        }
    }

    unsafe fn mut_to_ref(&self) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller.
            self.permits.mut_to_ref()
        }
    }

    unsafe fn drop_ref(&self) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller.
            self.permits.drop_ref()
        }
    }

    unsafe fn drop_mut(&self) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller.
            self.permits.drop_mut()
        }
    }
}

impl<B: Balance, P: Permits + WaitPermits> WaitPermits for SimpleAccount<B, P> {
    fn wait_ref(&self) {
        self.permits.wait_ref();
    }

    fn wait_mut(&self) {
        self.permits.wait_mut();
    }

    unsafe fn wait_ref_to_mut(&self) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller.
            self.permits.wait_ref_to_mut()
        }
    }
}
