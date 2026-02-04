use parking_lot::{RawRwLock as RwLock, lock_api::RawRwLock};

#[repr(transparent)]
pub(crate) struct Lock(RwLock);

impl Lock {
    pub(crate) fn new() -> Self {
        Lock(RwLock::INIT)
    }

    pub(crate) fn try_read(&self) -> Option<ReadCookie> {
        if self.0.try_lock_shared() {
            // SAFETY:
            // 1. Incremented just above.
            let cookie = unsafe { ReadCookie::new(&self.0) };
            Some(cookie)
        } else {
            None
        }
    }

    pub(crate) fn try_write(&self) -> Option<WriteCookie> {
        if self.0.try_lock_exclusive() {
            // SAFETY:
            // 1. Locked just above.
            let cookie = unsafe { WriteCookie::new(&self.0) };
            Some(cookie)
        } else {
            None
        }
    }
}

pub(crate) use limit_field_access::{ReadCookie, WriteCookie};

mod limit_field_access {
    // TODO: refactor once https://github.com/rust-lang/rust-project-goals/issues/273 passes.

    use parking_lot::lock_api::{RawRwLock, RawRwLockDowngrade, RawRwLockUpgrade};

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
    pub(crate) struct ReadCookie<'a> {
        lockref: &'a RwLock,
    }

    impl<'a> ReadCookie<'a> {
        /// Create a new read cookie.
        ///
        /// # Safety
        ///
        /// To safely call this function, the caller must ensure:
        ///
        /// 1. The lock pointed to has had its shared reference count incremented.
        pub(crate) unsafe fn new(lockref: &'a RwLock) -> Self {
            ReadCookie {
                // SAFETY:
                // 1. Ensured by caller.
                lockref,
            }
        }

        /// Try to upgrade this read cookie into a write cookie.
        pub(crate) fn try_upgrade(self) -> Result<WriteCookie<'a>, Self> {
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
                let cookie = unsafe { WriteCookie::new(lockref) };
                Ok(cookie)
            } else {
                Err(self)
            }
        }
    }

    impl<'a> Drop for ReadCookie<'a> {
        fn drop(&mut self) {
            unsafe {
                // SAFETY:
                // By invariant 1, there is at least one shared lock.
                self.lockref.unlock_shared();
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
    pub(crate) struct WriteCookie<'a> {
        lockref: &'a RwLock,
    }

    impl<'a> WriteCookie<'a> {
        /// Create a new read cookie.
        ///
        /// # Safety
        ///
        /// To safely call this function, the caller must ensure:
        ///
        /// 1. The referenced lock is locked for exclusive access.
        pub(crate) unsafe fn new(lockref: &'a RwLock) -> Self {
            WriteCookie {
                // SAFETY:
                // 1. Ensured by caller.
                lockref,
            }
        }

        /// Downgrade this write cookie to a read cookie of the same lock.
        pub(crate) fn downgrade(self) -> ReadCookie<'a> {
            let lockref = self.lockref;
            std::mem::forget(self);
            unsafe {
                // SAFETY:
                // The destructor didn't run on self, so the lock is still locked in exclusive fashion.
                lockref.downgrade();

                // SAFETY:
                // The call to downgrade ensures there is precisely 1 shared access.
                ReadCookie::new(lockref)
            }
        }
    }

    impl<'a> Drop for WriteCookie<'a> {
        fn drop(&mut self) {
            // SAFETY:
            // By invariant 1, an exclusive lock exists.
            unsafe {
                self.lockref.unlock_exclusive();
            }
        }
    }
}
