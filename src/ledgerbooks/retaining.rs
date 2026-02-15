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

impl<L: Ledger + Clone> LedgerBook<L> for RetainingBook<L> {
    type Storage = NonNull<L>;

    #[inline]
    fn free_list(&mut self) -> &mut Vec<Self::Storage> {
        &mut self.free
    }

    #[cfg(test)]
    fn expansions(&self) -> usize {
        self.expansions
    }

    #[cfg(test)]
    #[inline]
    fn count_allocation(&mut self) {
        self.total_allocations += 1;
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
        self.total_allocations = 0;
        self.expansions = 0;
        self.free.clear();
        self.dump.append(&mut self.alloc);
    }

    fn set_chunks(&mut self, chunk: usize, limit: usize) {
        self.chunk_size = chunk;
        self.max_chunk_size = limit;
    }

    #[inline]
    fn lazy_init(&mut self) {}

    #[inline]
    fn new_slice(&mut self, sample: &L) -> (&[L], &mut Vec<Self::Storage>) {
        let mut ledgers = Vec::with_capacity(self.chunk_size);
        for _ in 0..self.chunk_size {
            ledgers.push(sample.clone())
        }
        self.alloc.push(ledgers.into_boxed_slice());
        (self.alloc.last().unwrap(), &mut self.free)
    }

    #[cfg(test)]
    fn count_expansion(&mut self) {
        self.expansions += 1;
    }
}
