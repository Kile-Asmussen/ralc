use std::{num::NonZeroU64, ptr::NonNull};

use crate::{
    ledgerbooks::{CHUNK_SIZE, LedgerBook},
    ledgers::Ledger,
};

pub struct LeakyBook<L: Ledger + Default + 'static> {
    chunk_size: usize,
    max_chunk_size: usize,
    #[cfg(test)]
    total_allocations: usize,
    #[cfg(test)]
    expansions: usize,
    free: Vec<&'static L>,
}

impl<L: Ledger + Default + 'static> LeakyBook<L> {
    pub(crate) const fn new() -> Self {
        Self {
            chunk_size: CHUNK_SIZE,
            max_chunk_size: CHUNK_SIZE * CHUNK_SIZE,
            #[cfg(test)]
            expansions: 0,
            #[cfg(test)]
            total_allocations: 0,
            free: Vec::new(),
        }
    }
}

impl<L: Ledger + Default> LedgerBook<L> for LeakyBook<L> {
    unsafe fn deallocate(&mut self, ledger: NonNull<L>) {
        let ledger = unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            ledger.as_ref()
        };

        if ledger.reallocation() != NonZeroU64::MAX {
            self.free.push(ledger);
        }
    }

    fn extend_free_list(&mut self, vec: Vec<L>) {
        #[cfg(test)]
        {
            self.expansions += 1;
        }
        self.free.extend(Vec::leak(vec).iter())
    }

    fn next_free(&mut self) -> Option<NonNull<L>> {
        let res = self.free.pop().map(NonNull::from_ref);
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
        self.expansions = 0;
        self.chunk_size = CHUNK_SIZE;
        self.total_allocations = 0;
        self.free.clear();
    }
}
