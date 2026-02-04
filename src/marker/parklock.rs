use parking_lot::{RawRwLock as RwLock, lock_api::RawRwLock};

#[repr(transparent)]
pub(crate) struct ParkLock(RwLock);

impl ParkLock {
    pub(crate) fn new() -> Self {
        ParkLock(RwLock::INIT)
    }
}

impl CookieJar for ParkLock {
    type ReadToken<'a> = ParkReadToken<'a>;

    type WriteToken<'a> = ParkWriteToken<'a>;

    fn try_read(&self) -> Option<ParkReadToken<'_>> {
        if self.0.try_lock_shared() {
            let cookie = unsafe {
                // SAFETY:
                // 1. Incremented just above.
                ParkReadToken::new(&self.0)
            };
            Some(cookie)
        } else {
            None
        }
    }

    fn try_write(&self) -> Option<ParkWriteToken<'_>> {
        if self.0.try_lock_exclusive() {
            let cookie = unsafe {
                // SAFETY:
                // 1. Locked just above.
                ParkWriteToken::new(&self.0)
            };
            Some(cookie)
        } else {
            None
        }
    }
    fn read(&self) -> ParkReadToken<'_> {
        self.0.lock_shared();
        unsafe {
            // SAFETY:
            // 1. Locked just above.
            ParkReadToken::new(&self.0)
        }
    }

    fn write(&self) -> ParkWriteToken<'_> {
        self.0.lock_exclusive();
        unsafe {
            // SAFETY:
            // 1. Locked just above.
            ParkWriteToken::new(&self.0)
        }
    }
}

pub(crate) use limit_field_access::{ParkReadToken, ParkWriteToken};

use crate::marker::cookie::CookieJar;

mod limit_field_access {
    // TODO: refactor once https://github.com/rust-lang/rust-project-goals/issues/273 passes.

    use parking_lot::lock_api::{RawRwLock, RawRwLockDowngrade, RawRwLockUpgrade};

    use crate::marker::{
        cookie::{ReadCookie, WriteCookie},
        parklock::ParkLock,
    };

    use super::RwLock;

    /// Read-access cookie for a lock.
    ///
    /// # Safety
    ///
    /// The following invariants are upheld as long as this struct exists, and
    /// must be ensured prior construction:
    ///
    /// 1. The lock pointed to has a shared reference count at least equal to
    ///    the number of instances of this struct currently existing.
    #[repr(transparent)]
    pub(crate) struct ParkReadToken<'a> {
        lockref: &'a RwLock,
    }

    impl<'a> ParkReadToken<'a> {
        /// Create a new read cookie.
        ///
        /// # Safety
        ///
        /// To safely call this function, the caller must ensure:
        ///
        /// 1. The lock pointed to has had its shared reference count incremented.
        pub(crate) unsafe fn new(lockref: &'a RwLock) -> Self {
            ParkReadToken {
                // SAFETY:
                // 1. Ensured by caller.
                lockref,
            }
        }
    }

    impl<'a> Drop for ParkReadToken<'a> {
        fn drop(&mut self) {
            unsafe {
                // SAFETY:
                // By invariant 1 on self, there is at least one shared lock.
                self.lockref.unlock_shared();
            }
        }
    }

    impl<'a> Clone for ParkReadToken<'a> {
        fn clone(&self) -> Self {
            // By invariant 1 on self, this is not blocking
            self.lockref.lock_shared();
            unsafe {
                // SAFETY:
                // 1. we just incremented
                ParkReadToken::new(self.lockref)
            }
        }
    }

    impl<'a> ReadCookie for ParkReadToken<'a> {
        type UpgradesTo = ParkWriteToken<'a>;

        /// Try to upgrade this read cookie into a write cookie.
        fn try_upgrade(self) -> Result<ParkWriteToken<'a>, Self> {
            if self.lockref.try_lock_upgradable() {
                let lockref = self.lockref;
                std::mem::drop(self);
                unsafe {
                    // SAFETY:
                    // The upgradable lock was acquired above.
                    lockref.upgrade();
                }
                // SAFETY:
                // 1. the upgarde call above ensured the exclusive lock.
                let cookie = unsafe { ParkWriteToken::new(lockref) };
                Ok(cookie)
            } else {
                Err(self)
            }
        }
    }

    /// Write-access cookie for a lock.
    ///
    /// # Safety
    ///
    /// The following invariants are upheld as long as this struct exists, and
    /// must be ensured prior construction:
    ///
    /// 1. The lock pointed to is locked for exclusive access.
    #[repr(transparent)]
    pub(crate) struct ParkWriteToken<'a> {
        lockref: &'a RwLock,
    }

    impl<'a> ParkWriteToken<'a> {
        /// Create a new read cookie.
        ///
        /// # Safety
        ///
        /// To safely call this function, the caller must ensure:
        ///
        /// 1. The referenced lock is locked for exclusive access.
        pub(crate) unsafe fn new(lockref: &'a RwLock) -> Self {
            ParkWriteToken {
                // SAFETY:
                // 1. Ensured by caller.
                lockref,
            }
        }
    }

    impl<'a> Drop for ParkWriteToken<'a> {
        fn drop(&mut self) {
            // SAFETY:
            // By invariant 1, an exclusive lock exists.
            unsafe {
                self.lockref.unlock_exclusive();
            }
        }
    }

    impl<'a> WriteCookie for ParkWriteToken<'a> {
        type DowngradesTo = ParkReadToken<'a>;

        /// Downgrade this write cookie to a read cookie of the same lock.
        fn downgrade(self) -> ParkReadToken<'a> {
            let lockref = self.lockref;
            std::mem::forget(self);
            unsafe {
                // SAFETY:
                // The destructor didn't run on self, so the lock is still locked in exclusive fashion.
                lockref.downgrade();

                // SAFETY:
                // The call to downgrade ensures there is precisely 1 shared access.
                ParkReadToken::new(lockref)
            }
        }
    }
}
