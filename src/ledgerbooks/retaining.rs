use std::ptr::NonNull;

use crate::{
    ledgerbooks::{CHUNK_SIZE, LedgerBook},
    ledgers::Ledger,
};

pub struct RetainingBook<L: Ledger> {
    chunk_size: usize,
    max_chunk_size: usize,
    #[cfg(test)]
    expansions: usize,
    #[cfg(test)]
    total_allocations: usize,
    #[cfg(test)]
    dump: Vec<Box<[L]>>,
    alloc: Vec<Box<[L]>>,
    free: Vec<NonNull<L>>,
}

impl<L: Ledger> RetainingBook<L> {
    pub const fn new() -> Self {
        Self {
            chunk_size: CHUNK_SIZE,
            max_chunk_size: CHUNK_SIZE * CHUNK_SIZE,
            #[cfg(test)]
            expansions: 0,
            #[cfg(test)]
            total_allocations: 0,
            #[cfg(test)]
            dump: Vec::new(),
            alloc: Vec::new(),
            free: Vec::new(),
        }
    }
}

impl<L: Ledger> LedgerBook<L> for RetainingBook<L> {
    unsafe fn deallocate(&mut self, ledger: NonNull<L>) {
        self.free.push(ledger);
    }

    fn extend_free_list(&mut self, vec: Vec<L>) {
        #[cfg(test)]
        {
            self.expansions += 1;
        }
        self.alloc.push(vec.into_boxed_slice());
        let boxed_slice = &self.alloc.last().unwrap()[..];
        self.free.extend(boxed_slice.iter().map(NonNull::from_ref));
    }

    fn next_free(&mut self) -> Option<NonNull<L>> {
        let res = self.free.pop();
        #[cfg(test)]
        if res.is_some() {
            self.total_allocations += 1;
        }
        res
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn bump_chunk_size(&mut self) {
        if self.chunk_size < self.max_chunk_size {
            self.chunk_size *= 2;
        } else {
            self.chunk_size = self.max_chunk_size;
        }
    }

    #[cfg(test)]
    fn expansions(&self) -> usize {
        self.expansions
    }

    #[cfg(test)]
    fn total_allocations(&self) -> usize {
        self.total_allocations
    }

    #[cfg(test)]
    fn free_count(&self) -> usize {
        self.free.len()
    }

    #[cfg(test)]
    fn reset(&mut self) {
        self.chunk_size = CHUNK_SIZE;
        self.total_allocations = 0;
        self.expansions = 0;
        self.free.clear();
        self.dump.append(&mut self.alloc);
    }
}
