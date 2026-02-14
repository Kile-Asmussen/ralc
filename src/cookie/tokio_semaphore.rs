use tokio::sync::{Semaphore, SemaphorePermit};

use crate::cookie::{CookieJar, ReadCookie, WriteCookie};

impl CookieJar for Semaphore {
    type ReadToken<'a> = SmallSemaphorePermit<'a>;

    type WriteToken<'a> = LargeSemaphorePermit<'a>;

    #[cfg(test)]
    fn count(&self) -> u32 {
        self.available_permits() as u32
    }

    fn try_read(&self) -> Option<Self::ReadToken<'_>> {
        todo!()
    }

    fn try_write(&self) -> Option<Self::WriteToken<'_>> {
        todo!()
    }
}

pub struct SmallSemaphorePermit<'a>(&'a Semaphore, SemaphorePermit<'a>);

impl<'a> Clone for SmallSemaphorePermit<'a> {
    fn clone(&self) -> Self {
        Self(self.0, self.0.try_acquire().expect("Semaphore exhausted"))
    }
}

impl<'a> ReadCookie for SmallSemaphorePermit<'a> {
    type UpgradesTo = LargeSemaphorePermit<'a>;

    fn try_upgrade(self) -> Result<Self::UpgradesTo, Self> {
        if let Ok(mut permit) = self.0.try_acquire_many(u32::MAX - 1) {
            permit.merge(self.1);
            Ok(LargeSemaphorePermit(self.0, permit))
        } else {
            Err(self)
        }
    }
}

pub struct LargeSemaphorePermit<'a>(&'a Semaphore, SemaphorePermit<'a>);

impl<'a> WriteCookie for LargeSemaphorePermit<'a> {
    type DowngradesTo = SmallSemaphorePermit<'a>;

    fn downgrade(mut self) -> Self::DowngradesTo {
        SmallSemaphorePermit(self.0, self.1.split(1).unwrap())
    }
}
