use std::ptr::NonNull;

use crate::ledgers::Ledger;
#[cfg(feature = "bumpalo")]
mod bumpalo;
mod leaking;
mod retaining;

#[cfg(feature = "bumpalo")]
pub use bumpalo::BumpyBook;
pub use leaking::LeakyBook;
pub use retaining::RetainingBook;

#[cfg(not(test))]
const CHUNK_SIZE: usize = 1024;

#[cfg(test)]
const CHUNK_SIZE: usize = 16;

pub trait LedgerBook<L: Ledger> {
    /// # Safety requirements
    /// 1. `ledger` is a return from next_free
    /// 2. `free` is not called for the same pointer twice
    unsafe fn deallocate(&mut self, ledger: NonNull<L>);

    fn allocate<F: FnMut() -> L>(&mut self, produce: F) -> NonNull<L>;

    #[cfg(test)]
    fn expansions(&self) -> usize;

    #[cfg(test)]
    fn total_allocations(&self) -> usize;

    #[cfg(test)]
    fn free_count(&self) -> usize;

    #[cfg(test)]
    fn reset(&mut self);
}
