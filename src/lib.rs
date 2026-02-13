use crate::ledgers::{GlobalLedger, Ledger, LocalLedger};

mod borrowed;
pub mod cookie;
pub mod ledgers;
mod owned;
mod read;
#[cfg(test)]
mod test;
mod write;

pub use borrowed::BorrowRalc;
pub use owned::OwnedRalc;
pub use read::ReadRalc;
pub use write::WriteRalc;

pub enum Ralc<T, L: Ledger> {
    Borrow(BorrowRalc<T, L>),
    Owned(OwnedRalc<T, L>),
}

impl<T, L: Ledger> Ralc<T, L> {
    pub fn is_owned(&self) -> bool {
        match self {
            Self::Borrow(_borrow_ralc) => false,
            Self::Owned(_owned_ralc) => true,
        }
    }
}

impl<T: Send + Sync> Ralc<T, GlobalLedger> {
    pub fn new_global(data: T) -> Self {
        Self::Owned(OwnedRalc::new_global(data))
    }
}

impl<T> Ralc<T, LocalLedger> {
    pub fn new_local(data: T) -> Self {
        Self::Owned(OwnedRalc::new_local(data))
    }
}

impl<T, L: Ledger> Ralc<T, L> {
    pub fn check(&self) -> bool {
        match self {
            Self::Borrow(borrow_ralc) => borrow_ralc.check(),
            Self::Owned(_owned_ralc) => true,
        }
    }

    pub fn try_read(&self) -> Option<ReadRalc<'_, T, L>> {
        match self {
            Self::Borrow(borrow_ralc) => borrow_ralc.try_read(),
            Self::Owned(owned_ralc) => owned_ralc.try_read(),
        }
    }

    pub fn try_write(&self) -> Option<WriteRalc<'_, T, L>> {
        match self {
            Self::Borrow(borrow_ralc) => borrow_ralc.try_write(),
            Self::Owned(owned_ralc) => owned_ralc.try_write(),
        }
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

impl<T, L: Ledger> From<OwnedRalc<T, L>> for Ralc<T, L> {
    fn from(value: OwnedRalc<T, L>) -> Self {
        Ralc::Owned(value)
    }
}

impl<T, L: Ledger> From<BorrowRalc<T, L>> for Ralc<T, L> {
    fn from(value: BorrowRalc<T, L>) -> Self {
        Ralc::Borrow(value)
    }
}
