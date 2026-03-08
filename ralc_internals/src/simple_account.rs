use std::{cell::Cell, sync::atomic::AtomicU64};

use parking_lot::RawRwLock;

use crate::accounts::{balances::Balance, permits::Permits};

pub struct SimpleAccount<B: Balance, P: Permits> {
    balance: B,
    permits: P,
}

pub type SyncAccount = SimpleAccount<AtomicU64, RawRwLock>;
pub type CellAccount = SimpleAccount<Cell<u64>, Cell<u32>>;

// SAFETY:
// 1. delegated implementation.
unsafe impl<B: Balance, P: Permits> Balance for SimpleAccount<B, P> {
    fn invalidate(&self) {
        self.balance.invalidate();
    }

    fn check(&self) -> u64 {
        self.balance.check()
    }
}

// SAFETY:
// 1. delegated implementation.
unsafe impl<B: Balance, P: Permits> Permits for SimpleAccount<B, P> {
    fn try_reference(&self) -> bool {
        self.permits.try_reference()
    }

    fn try_mutation(&self) -> bool {
        self.permits.try_mutation()
    }

    unsafe fn try_escalate(&self) -> bool {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            self.permits.try_escalate()
        }
    }

    fn reference_permit(&self) -> bool {
        self.permits.reference_permit()
    }

    fn mutation_permit(&self) -> bool {
        self.permits.mutation_permit()
    }

    unsafe fn relax_permit(&self) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            self.permits.relax_permit()
        }
    }

    unsafe fn abandon_reference(&self) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            self.permits.abandon_reference()
        }
    }

    unsafe fn abandon_mutation(&self) {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            self.permits.abandon_mutation()
        }
    }
}
