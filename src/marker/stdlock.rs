use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::marker::cookie::{CookieJar, ReadCookie, WriteCookie};

pub(crate) struct StdLock {
    mutex: Mutex<()>,
    lock: RwLock<()>,
}

impl StdLock {
    pub(crate) fn new() -> Self {
        StdLock {
            mutex: Mutex::new(()),
            lock: RwLock::new(()),
        }
    }
}

impl CookieJar for StdLock {
    fn try_read(&self) -> Option<StdReadToken<'_>> {
        if let Ok(handle) = self.lock.try_read() {
            Some(StdReadToken {
                mutexref: &self.mutex,
                lockref: &self.lock,
                handle,
            })
        } else {
            None
        }
    }

    fn try_write(&self) -> Option<StdWriteToken<'_>> {
        if let Ok(handle) = self.lock.try_write() {
            Some(StdWriteToken {
                mutexref: &self.mutex,
                lockref: &self.lock,
                handle,
            })
        } else {
            None
        }
    }

    type ReadToken<'a> = StdReadToken<'a>;

    type WriteToken<'a> = StdWriteToken<'a>;

    fn read(&self) -> StdReadToken<'_> {
        StdReadToken {
            lockref: &self.lock,
            mutexref: &self.mutex,
            handle: self.lock.read().expect("Failed to aquire read lock"),
        }
    }

    fn write(&self) -> StdWriteToken<'_> {
        StdWriteToken {
            lockref: &self.lock,
            mutexref: &self.mutex,
            handle: self.lock.write().expect("Failed to aquire read lock"),
        }
    }
}

pub(crate) struct StdReadToken<'a> {
    mutexref: &'a Mutex<()>,
    lockref: &'a RwLock<()>,
    handle: RwLockReadGuard<'a, ()>,
}

impl<'a> ReadCookie for StdReadToken<'a> {
    /// Try to upgrade this read cookie into a write cookie.
    fn try_upgrade(self) -> Result<StdWriteToken<'a>, Self> {
        if let Ok(_) = self.mutexref.try_lock() {
            let handle = self.handle;
            let lockref = self.lockref;
            let mutexref = self.mutexref;
            std::mem::drop(handle);

            if let Ok(handle) = lockref.try_write() {
                Ok(StdWriteToken {
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

    type UpgradesTo = StdWriteToken<'a>;
}

impl<'a> Clone for StdReadToken<'a> {
    fn clone(&self) -> Self {
        Self {
            mutexref: self.mutexref,
            lockref: self.lockref,
            handle: self
                .lockref
                .read()
                .expect("Unable to acquire reading rights"),
        }
    }
}

pub(crate) struct StdWriteToken<'a> {
    lockref: &'a RwLock<()>,
    mutexref: &'a Mutex<()>,
    handle: RwLockWriteGuard<'a, ()>,
}

impl<'a> WriteCookie for StdWriteToken<'a> {
    /// Downgrade this write cookie to a read cookie of the same lock.
    fn downgrade(self) -> StdReadToken<'a> {
        let lockref = self.lockref;
        let handle = self.handle;
        let mutexref = self.mutexref;

        let _guard = mutexref.lock().expect("Unable to lock mutex");
        std::mem::drop(handle);
        let handle = lockref.read().expect("Unable to acquire reading rights");

        StdReadToken {
            mutexref,
            lockref,
            handle,
        }
    }

    type DowngradesTo = StdReadToken<'a>;
}
