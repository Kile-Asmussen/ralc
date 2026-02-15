use std::marker::PhantomData;

use bumpalo::Bump;

use crate::{
    ledgerbooks::{CHUNK_SIZE, LedgerBook, RefLike},
    ledgers::Ledger,
};

pub struct BumpyBook<R: RefLike<L>, L: Ledger + Clone + 'static> {
    chunk_size: usize,
    max_chunk_size: usize,
    #[cfg(test)]
    total_allocations: usize,
    #[cfg(test)]
    expansions: usize,
    #[cfg(test)]
    dump: Vec<Bump<8>>,
    alo: Option<Bump<8>>,
    free: Vec<R>,
    _phantom: PhantomData<L>,
}

impl<R, L> BumpyBook<R, L>
where
    R: RefLike<L>,
    L: Ledger + Clone + 'static,
{
    pub(crate) const fn new() -> Self {
        Self {
            chunk_size: CHUNK_SIZE,
            max_chunk_size: CHUNK_SIZE,
            #[cfg(test)]
            expansions: 0,
            #[cfg(test)]
            total_allocations: 0,
            #[cfg(test)]
            dump: Vec::new(),
            alo: None,
            free: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<R, L> LedgerBook<L> for BumpyBook<R, L>
where
    R: RefLike<L>,
    L: Ledger + Clone,
{
    type Storage = R;

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
        if let Some(alo) = self.alo.take() {
            self.dump.push(alo);
        }
        self.free.clear();
    }

    #[inline]
    fn set_chunks(&mut self, chunk: usize, limit: usize) {
        self.chunk_size = chunk;
        self.max_chunk_size = limit;
    }

    #[cfg(test)]
    #[inline]
    fn count_allocation(&mut self) {
        self.total_allocations += 1;
    }

    #[inline]
    fn free_list(&mut self) -> &mut Vec<Self::Storage> {
        &mut self.free
    }

    #[inline]
    fn lazy_init(&mut self) {
        if self.alo.is_none() {
            self.alo = Some(Bump::with_min_align());
        };
    }

    #[inline]
    fn new_slice(&mut self, sample: &L) -> (&[L], &mut Vec<Self::Storage>) {
        (
            self.alo
                .as_ref()
                .unwrap()
                .alloc_slice_fill_clone(self.chunk_size, sample),
            &mut self.free,
        )
    }

    #[cfg(test)]
    #[inline]
    fn count_expansion(&mut self) {
        self.expansions += 1;
    }
}
