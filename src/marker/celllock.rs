use std::{cell::Cell, u32};

use crate::marker::cookie::{CookieJar, ReadCookie, WriteCookie};

impl CookieJar for Cell<u32> {
    type ReadToken<'a> = CellDecr<'a>;

    type WriteToken<'a> = CellZero<'a>;

    fn try_read(&self) -> Option<Self::ReadToken<'_>> {
        if self.get() < u32::MAX {
            self.update(|n| n + 1);
            Some(CellDecr { cell: self })
        } else {
            None
        }
    }

    fn read(&self) -> Self::ReadToken<'_> {
        self.try_read().unwrap()
    }

    fn try_write(&self) -> Option<Self::WriteToken<'_>> {
        if self.get() == 0 {
            self.update(|n| n + 1);
            Some(CellZero { cell: self })
        } else {
            None
        }
    }

    fn write(&self) -> Self::WriteToken<'_> {
        self.try_write().unwrap()
    }
}

pub struct CellDecr<'a> {
    cell: &'a Cell<u32>,
}

impl<'a> Clone for CellDecr<'a> {
    fn clone(&self) -> Self {
        self.cell.update(|n| n + 1);
        CellDecr { cell: self.cell }
    }
}

impl<'a> ReadCookie for CellDecr<'a> {
    type UpgradesTo = CellZero<'a>;

    fn try_upgrade(self) -> Result<Self::UpgradesTo, Self> {
        if self.cell.get() == 1 {
            let cell = self.cell;
            std::mem::drop(self);
            cell.set(u32::MAX);
            Ok(CellZero { cell })
        } else {
            Err(self)
        }
    }
}

impl<'a> Drop for CellDecr<'a> {
    fn drop(&mut self) {
        self.cell.update(|n| n - 1);
    }
}

pub struct CellZero<'a> {
    cell: &'a Cell<u32>,
}

impl<'a> WriteCookie for CellZero<'a> {
    type DowngradesTo = CellDecr<'a>;

    fn downgrade(self) -> Self::DowngradesTo {
        let cell = self.cell;
        std::mem::drop(self);
        cell.set(1);
        CellDecr { cell }
    }
}
