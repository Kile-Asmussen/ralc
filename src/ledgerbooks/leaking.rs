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

impl<L: Ledger + Clone> LeakyBook<L> {
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

impl<L: Ledger + Clone> LedgerBook<L> for LeakyBook<L> {
    type Storage = &'static L;

    #[inline]
    fn free_list(&mut self) -> &mut Vec<Self::Storage> {
        &mut self.free
    }

    #[inline]
    fn lazy_init(&mut self) {}

    #[inline]
    fn new_slice(&mut self, sample: &L) -> (&[L], &mut Vec<Self::Storage>) {
        let mut ledgers = Vec::with_capacity(self.chunk_size);
        for _ in 0..self.chunk_size {
            ledgers.push(sample.clone())
        }
        (ledgers.leak(), &mut self.free)
    }

    #[cfg(test)]
    #[inline]
    fn count_expansion(&mut self) {
        self.expansions += 1;
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
        self.total_allocations = 0;
        self.free.clear();
    }

    fn set_chunks(&mut self, chunk: usize, limit: usize) {
        self.chunk_size = chunk;
        self.max_chunk_size = limit;
    }

    #[cfg(test)]
    #[inline]
    fn count_allocation(&mut self) {
        self.total_allocations += 1;
    }
}
