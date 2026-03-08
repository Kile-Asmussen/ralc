use parking_lot::lock_api::{RawRwLock, RawRwLockDowngrade, RawRwLockUpgrade};

use crate::accounts::permits::Permits;

unsafe impl Permits for parking_lot::RawRwLock {
    #[inline]
    fn try_reference(&self) -> bool {
        self.try_lock_shared()
    }

    #[inline]
    fn try_mutation(&self) -> bool {
        self.try_lock_shared()
    }

    #[inline]
    unsafe fn try_escalate(&self) -> bool {
        if self.try_lock_upgradable() {
            unsafe {
                // SAFETY:
                // Guaranteed by caller
                self.unlock_shared();
            }

            let upgraded = unsafe {
                // SAFETY:
                // See above
                self.try_upgrade()
            };

            if upgraded {
                return true;
            } else {
                self.lock_shared();

                unsafe {
                    // SAFEY:
                    // See above
                    self.unlock_upgradable();
                }
            }
        }

        false
    }

    fn reference_permit(&self) -> bool {
        self.lock_shared();
        true
    }

    fn mutation_permit(&self) -> bool {
        self.lock_exclusive();
        true
    }

    unsafe fn relax_permit(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller
            self.downgrade();
        }
    }

    unsafe fn abandon_reference(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller
            self.unlock_shared();
        }
    }

    unsafe fn abandon_mutation(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller
            self.unlock_exclusive();
        }
    }
}
