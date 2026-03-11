mod ledgers;

use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
}; 

use parking_lot::{Mutex, lock_api::{
    RawRwLock, RawRwLockDowngrade, RawRwLockUpgrade, RawRwLockUpgradeDowngrade,
}};
use ralc_internals::{
    accounts::{freeable::Freeable, permits::Permits},
    delegate_impl::DelegateAccountImpl,
};

static GLOBAL_ALLOCATOR : Mutex<Allocator<GlobalAccount>> = Mutex::new()

struct GlobalAccount(AtomicU64, ParkingLock);

impl Default for GlobalAccount {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

fn alloc() -> &'static GlobalAccount {
    let mut res: &'static GlobalAccount;
    loop {
        let free = FREE_LIST.load(Ordering::Relaxed);
        res = free.as_ref()
    }
}

impl DelegateAccountImpl for &'static GlobalAccount {
    type DelegatedBalance = AtomicU64;

    type DelegatedPermits = ParkingLock;

    fn balance(&self) -> &Self::DelegatedBalance {
        &self.0
    }

    fn permits(&self) -> &Self::DelegatedPermits {
        &self.1
    }
}

unsafe impl Freeable for GlobalAccount {
    unsafe fn free(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller.
            self.abandon_mutation();
        }
        // IMPL SAFETY:
        // 1. See above
    }
}

#[repr(transparent)]
struct ParkingLock(parking_lot::RawRwLock);

impl Default for ParkingLock {
    fn default() -> Self {
        Self(parking_lot::RawRwLock::INIT)
    }
}

unsafe impl Permits for ParkingLock {
    type UnderlyingLockableEntity = parking_lot::RawRwLock;

    unsafe fn underlying(&self) -> &Self::UnderlyingLockableEntity {
        &self.0
    }

    fn try_reference(&self) -> bool {
        self.0.try_lock_shared()
    }

    fn try_mutation(&self) -> bool {
        self.0.try_lock_exclusive()
    }

    unsafe fn try_escalate(&self) -> bool {
        if self.0.try_lock_upgradable() {
            unsafe {
                self.0.unlock_shared();
            }

            let upgraded = unsafe { self.0.try_upgrade() };

            if upgraded {
                return true;
            }

            unsafe {
                self.0.downgrade_upgradable();
            }
        }
        false
    }

    unsafe fn relax_permit(&self) {
        unsafe {
            self.0.downgrade();
        }
    }

    unsafe fn abandon_reference(&self) {
        unsafe {
            self.0.unlock_shared();
        }
    }

    unsafe fn abandon_mutation(&self) {
        unsafe {
            self.0.unlock_exclusive();
        }
    }
}
