use std::{num::NonZeroU64, ptr::NonNull};

use crate::{ledgerbooks::LedgerBook, ledgers::Ledger};

pub struct LeakyBook<L: Ledger + Default + 'static> {
    chunk_size: usize,
    max_chunk_size: usize,
    free: Vec<&'static L>,
}

impl<L: Ledger + Default + 'static> LeakyBook<L> {
    pub(crate) const fn new() -> Self {
        Self {
            chunk_size: 1024,
            max_chunk_size: 1024 * 1024,
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
        self.free.extend(Vec::leak(vec).iter())
    }

    fn next_free(&mut self) -> Option<NonNull<L>> {
        self.free.pop().map(NonNull::from_ref)
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
}
