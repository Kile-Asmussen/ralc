use std::{cell::Cell, u32};

#[cfg(test)]
use assert_impl::assert_impl;

use crate::cookie::{CookieJar, ReadCookie, WriteCookie};

impl CookieJar for Cell<u32> {
    type ReadToken<'a> = CellDecr<'a>;

    type WriteToken<'a> = CellZero<'a>;

    fn try_read(&self) -> Option<Self::ReadToken<'_>> {
        let n = self.get();
        if n < u32::MAX {
            self.set(n + 1);
            Some(CellDecr { cell: self })
        } else {
            None
        }
    }

    fn try_write(&self) -> Option<Self::WriteToken<'_>> {
        if self.get() == 0 {
            self.set(u32::MAX);
            Some(CellZero { cell: self })
        } else {
            None
        }
    }

    #[cfg(test)]
    fn count(&self) -> u32 {
        self.get()
    }

    const INIT: Self = Cell::new(0);
}

#[repr(transparent)]
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

#[repr(transparent)]
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

impl<'a> Drop for CellZero<'a> {
    fn drop(&mut self) {
        self.cell.set(0);
    }
}

#[test]
fn marker_trait_impls() {
    assert_impl!(!Send: CellDecr<'static>, CellZero<'static>);
    assert_impl!(!Sync: CellDecr<'static>, CellZero<'static>);
}
