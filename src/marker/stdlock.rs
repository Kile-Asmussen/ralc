use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub(crate) struct Lock {
    mutex: Mutex<()>,
    lock: RwLock<()>,
}

impl Lock {
    pub(crate) fn new() -> Self {
        Lock {
            mutex: Mutex::new(()),
            lock: RwLock::new(()),
        }
    }

    pub(crate) fn try_read(&self) -> Option<ReadCookie<'_>> {
        if let Ok(handle) = self.lock.try_read() {
            Some(ReadCookie {
                mutexref: &self.mutex,
                lockref: &self.lock,
                handle,
            })
        } else {
            None
        }
    }

    pub(crate) fn try_write(&self) -> Option<WriteCookie<'_>> {
        if let Ok(handle) = self.lock.try_write() {
            Some(WriteCookie {
                mutexref: &self.mutex,
                lockref: &self.lock,
                handle,
            })
        } else {
            None
        }
    }
}

pub(crate) struct ReadCookie<'a> {
    mutexref: &'a Mutex<()>,
    lockref: &'a RwLock<()>,
    handle: RwLockReadGuard<'a, ()>,
}

impl<'a> ReadCookie<'a> {
    /// Try to upgrade this read cookie into a write cookie.
    pub(crate) fn try_upgrade(self) -> Result<WriteCookie<'a>, Self> {
        if let Ok(_) = self.mutexref.try_lock() {
            let handle = self.handle;
            let lockref = self.lockref;
            let mutexref = self.mutexref;
            std::mem::drop(handle);

            if let Ok(handle) = lockref.try_write() {
                Ok(WriteCookie {
                    lockref,
                    mutexref,
                    handle,
                })
            } else {
                Err(Self {
                    mutexref,
                    lockref,
                    handle: lockref.read().expect("Unable to re-acquire reading rights"),
                })
            }
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
pub(crate) struct WriteCookie<'a> {
    lockref: &'a RwLock<()>,
    mutexref: &'a Mutex<()>,
    handle: RwLockWriteGuard<'a, ()>,
}

impl<'a> WriteCookie<'a> {
    /// Downgrade this write cookie to a read cookie of the same lock.
    pub(crate) fn downgrade(self) -> ReadCookie<'a> {
        let lockref = self.lockref;
        let handle = self.handle;
        let mutexref = self.mutexref;

        let _guard = mutexref.lock().expect("Unable to lock mutex");
        std::mem::drop(handle);
        let handle = lockref.read().expect("Unable to acquire read lock");

        ReadCookie {
            mutexref,
            lockref,
            handle,
        }
    }
}
