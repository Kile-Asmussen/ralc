use std::{cell::Cell, num::NonZeroU32};

use parking_lot::{
    RawRwLock as RwLock,
    lock_api::{RawRwLock, RawRwLockDowngrade, RawRwLockUpgrade, RawRwLockUpgradeDowngrade},
};

pub trait Permits {
    const INIT: Self;

    fn try_ref(&self) -> bool;
    fn try_mut(&self) -> bool;

    /// # Safety requirements:
    /// 1. Precondition: [`Self::try_ref`] must have been called
    ///    returning `true`, OR [`WaitableGate::wait_ref`] must have returned.
    unsafe fn try_ref_to_mut(&self) -> bool;

    /// # Safety requirements:
    /// 1. Precondition: [`Self::try_mut`] must have been called
    ///    returning `true`, OR [`WaitableGate::wait_mut`] must have returned.
    unsafe fn mut_to_ref(&self);

    /// # Safety requirements:
    /// 1. Precondition: [`Self::try_ref`] must have been called
    ///    returning `true`, OR [`WaitableGate::wait_ref`] must have returned.
    unsafe fn drop_ref(&self);

    /// # Safety requirements:
    /// 1. Precondition: [`Self::try_mut`] must have been called
    ///    returning `true`, OR [`WaitableGate::wait_mut`] must have returned.
    unsafe fn drop_mut(&self);
}

pub trait WaitPermits: Permits {
    fn wait_ref(&self);
    fn wait_mut(&self);

    /// # Safety requirements:
    /// 1. Precondition: [`Self::try_mut`] must have been called
    ///    returning `true`, OR [`WaitableGate::wait_mut`] must have returned.
    unsafe fn wait_ref_to_mut(&self);
}

impl Permits for Cell<u32> {
    fn try_ref(&self) -> bool {
        self.get().checked_add(1).is_some_and(|n| {
            self.set(n);
            true
        })
    }

    fn try_mut(&self) -> bool {
        self.get().checked_add(u32::MAX).is_some_and(|_| {
            self.set(u32::MAX);
            true
        })
    }

    unsafe fn drop_ref(&self) {
        self.set(self.get() - 1);
    }

    unsafe fn drop_mut(&self) {
        self.set(0);
    }

    unsafe fn try_ref_to_mut(&self) -> bool {
        self.get().checked_add(u32::MAX - 1).is_some_and(|_| {
            self.set(u32::MAX);
            true
        })
    }

    unsafe fn mut_to_ref(&self) {
        self.set(1);
    }

    const INIT: Self = Cell::new(1);
}

impl Permits for RwLock {
    fn try_ref(&self) -> bool {
        self.try_lock_shared()
    }

    fn try_mut(&self) -> bool {
        self.try_lock_exclusive()
    }

    unsafe fn try_ref_to_mut(&self) -> bool {
        unsafe {
            if self.try_lock_upgradable() {
                // SAFETY:
                // Guaranteed by caller (1)
                self.unlock_shared();

                // SAFETY:
                // See above
                if self.try_upgrade() {
                    return true;
                } else {
                    // SAFETY:
                    // See above
                    self.downgrade_upgradable();
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    unsafe fn mut_to_ref(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller (1)
            self.downgrade();
        }
    }

    unsafe fn drop_ref(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller (1)
            self.unlock_shared();
        }
    }

    unsafe fn drop_mut(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller (1)
            self.unlock_exclusive();
        }
    }

    const INIT: Self = <RwLock as RawRwLock>::INIT;
}

impl WaitPermits for RwLock {
    unsafe fn wait_ref_to_mut(&self) {
        self.lock_upgradable();
        unsafe {
            // SAFETY:
            // Guaranteed by caller (1)
            self.unlock_shared();
            // SAFETY:
            // See above
            self.upgrade();
        }
    }

    fn wait_ref(&self) {
        self.lock_shared();
    }

    fn wait_mut(&self) {
        self.lock_exclusive();
    }
}
