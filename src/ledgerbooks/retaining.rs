use std::ptr::NonNull;

use crate::{ledgerbooks::LedgerBook, ledgers::Ledger};

pub struct RetainingBook<L: Ledger> {
    max_chunk_size: usize,
    alloc: Vec<Vec<L>>,
    free: Vec<NonNull<L>>,
}

impl<L: Ledger> RetainingBook<L> {
    pub const fn new() -> Self {
        Self {
            max_chunk_size: 1024 * 1024,
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
        self.alloc.push(vec);
        let vec = self.alloc.last().unwrap();
        self.free.extend(vec.iter().map(NonNull::from_ref));
    }

    fn next_free(&mut self) -> Option<NonNull<L>> {
        self.free.pop()
    }

    fn chunk_size(&self) -> usize {
        self.alloc
            .last()
            .map(|v| usize::max(v.len() * 2, self.max_chunk_size))
            .unwrap_or(1024)
    }

    fn bump_chunk_size(&mut self) {}
}
