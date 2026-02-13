use crate::ledgers::Ledger;

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

pub enum Racl<T, L: Ledger> {
    Borrow(BorrowRalc<T, L>),
    Owned(OwnedRalc<T, L>),
}

impl<T, L: Ledger> Racl<T, L> {
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

impl<T, L: Ledger> Clone for Racl<T, L> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrow(borrow_ralc) => Self::Borrow(*borrow_ralc),
            Self::Owned(owned_ralc) => Self::Borrow(owned_ralc.borrow()),
        }
    }
}
