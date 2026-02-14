use std::ptr::NonNull;

use crate::ledgers::Ledger;
mod leaking;
mod retaining;

pub use leaking::LeakyBook;
pub use retaining::RetainingBook;

pub trait LedgerBook<L: Ledger> {
    /// # Safety requirements
    /// 1. `ledger` is a return from next_free
    /// 2. `free` is not called for the same pointer twice
    unsafe fn deallocate(&mut self, ledger: NonNull<L>);

    fn extend_free_list(&mut self, vec: Vec<L>);
    fn next_free(&mut self) -> Option<NonNull<L>>;

    fn chunk_size(&self) -> usize {
        1024
    }

    fn bump_chunk_size(&mut self) {}

    fn allocate<F: FnMut() -> L>(&mut self, mut produce: F) -> NonNull<L> {
        if let Some(x) = self.next_free() {
            return x;
        } else {
            let size = self.chunk_size();
            let mut ledgers = Vec::with_capacity(size);
            for _ in 0..size {
                ledgers.push(produce())
            }
            self.bump_chunk_size();

            self.extend_free_list(ledgers);

            return self.next_free().unwrap();
        }
    }
}
