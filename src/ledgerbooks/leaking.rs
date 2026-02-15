use std::{num::NonZeroU64, ptr::NonNull};

use crate::{
    ledgerbooks::{CHUNK_SIZE, LedgerBook},
    ledgers::Ledger,
};

pub struct LeakyBook<L: Ledger + 'static> {
    chunk_size: usize,
    max_chunk_size: usize,
    #[cfg(test)]
    total_allocations: usize,
    #[cfg(test)]
    expansions: usize,
    free: Vec<&'static L>,
}

impl<L: Ledger + Default + 'static> LeakyBook<L> {
    #[cfg_attr(feature = "bumpalo", allow(dead_code))]
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

impl<L: Ledger> LedgerBook<L> for LeakyBook<L> {
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

    fn allocate<F: FnMut() -> L>(&mut self, mut produce: F) -> NonNull<L> {
        #[cfg(test)]
        {
            self.total_allocations += 1;
        }
        let res = self.free.pop().map(NonNull::from_ref);
        if let Some(x) = res {
            return x;
        } else {
            let mut ledgers = Vec::with_capacity(self.chunk_size);
            for _ in 0..self.chunk_size {
                ledgers.push(produce())
            }
            self.chunk_size = usize::max(self.chunk_size * 2, self.max_chunk_size);

            #[cfg(test)]
            {
                self.expansions += 1;
            }
            let leak = ledgers.leak();
            self.free.extend(leak.iter());

            return self.free.pop().map(NonNull::from_ref).unwrap();
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
