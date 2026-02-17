use std::fmt;

mod aapointers;
pub mod accounts;
mod borrowed;
pub mod cookie;
pub mod ledgerbooks;
pub mod ledgers;

mod mut_;
mod owned;
mod ref_;
#[cfg(test)]
mod test;

pub use borrowed::BorrowRalc;
pub use mut_::RalcMut;
pub use owned::OwnedRalc;
pub use ref_::RalcRef;

use crate::ledgers::Ledger;

pub mod allocators;

/// Reallocation-counting smart pointer
///
/// Analogous to [`Cow`](std::borrow::Cow), this type implements a borrowed and owned
/// variant. Differently, these two interplay freely.
///
/// Calling [`.clone()`](std::clone::Clone::clone) will return a borrowed reference

pub enum Ralc<T, L: Ledger> {
    Borrow(BorrowRalc<T, L>),
    Owned(OwnedRalc<T, L>),
}

impl<T, L: Ledger> From<BorrowRalc<T, L>> for Ralc<T, L> {
    fn from(value: BorrowRalc<T, L>) -> Self {
        Self::Borrow(value)
    }
}

impl<T, L: Ledger> From<OwnedRalc<T, L>> for Ralc<T, L> {
    fn from(value: OwnedRalc<T, L>) -> Self {
        Self::Owned(value)
    }
}

impl<T, L: Ledger> Clone for Ralc<T, L> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrow(borrow_ralc) => Self::Borrow(*borrow_ralc),
            Self::Owned(owned_ralc) => Self::Borrow(owned_ralc.borrow()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum NoAccess {
    /// Reference is currently unavailable due to an active lock.
    Blocked,
    /// Reference is stale.
    Stale,
    /// Reference is currently unavailable due to an active lock,
    /// and a waiting operation was requested which would deadlock.
    Deadlock,
}

pub type Result<T> = std::result::Result<T, NoAccess>;

impl<T, L: Ledger> Ralc<T, L> {
    pub fn take(&mut self) -> Self {
        let mut res = self.clone();
        std::mem::swap(self, &mut res);
        res
    }

    pub fn is_owned(&self) -> bool {
        match self {
            Self::Borrow(_borrow_ralc) => false,
            Self::Owned(_owned_ralc) => true,
        }
    }

    pub fn check(&self) -> bool {
        match self {
            Self::Borrow(borrow_ralc) => borrow_ralc.check(),
            Self::Owned(_owned_ralc) => true,
        }
    }

    pub fn try_read(&self) -> Result<RalcRef<'_, T, L>> {
        match self {
            Self::Borrow(borrow_ralc) => borrow_ralc.try_read(),
            Self::Owned(owned_ralc) => owned_ralc.try_read(),
        }
    }

    pub fn try_write(&self) -> Result<RalcMut<'_, T, L>> {
        match self {
            Self::Borrow(borrow_ralc) => borrow_ralc.try_write(),
            Self::Owned(owned_ralc) => owned_ralc.try_write(),
        }
    }

    pub fn read(&self) -> Result<RalcRef<'_, T, L>> {
        match self {
            Self::Borrow(borrow_ralc) => borrow_ralc.read(),
            Self::Owned(owned_ralc) => owned_ralc.read(),
        }
    }

    pub fn write(&self) -> Result<RalcMut<'_, T, L>> {
        match self {
            Self::Borrow(borrow_ralc) => borrow_ralc.write(),
            Self::Owned(owned_ralc) => owned_ralc.write(),
        }
    }
}

impl<T: fmt::Display, L: Ledger> fmt::Display for Ralc<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrow(borrow_ralc) => fmt::Display::fmt(borrow_ralc, f),
            Self::Owned(owned_ralc) => fmt::Display::fmt(owned_ralc, f),
        }
    }
}

impl<T: fmt::Debug, L: Ledger> fmt::Debug for Ralc<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrow(borrow_ralc) => fmt::Debug::fmt(borrow_ralc, f),
            Self::Owned(owned_ralc) => fmt::Debug::fmt(owned_ralc, f),
        }
    }
}
