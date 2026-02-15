use std::{marker::PhantomData, num::NonZeroU64, ptr::NonNull};

use bumpalo::Bump;

use crate::{ledgerbooks::LedgerBook, ledgers::Ledger};

pub trait RefLike<L>: Copy + Sized {
    fn to_ptr(self) -> NonNull<L>;
    fn from_ptr(ptr: NonNull<L>) -> Self;
}

impl<L> RefLike<L> for NonNull<L> {
    fn to_ptr(self) -> NonNull<L> {
        self
    }

    fn from_ptr(ptr: NonNull<L>) -> Self {
        ptr
    }
}

impl<L> RefLike<L> for &'static L {
    fn to_ptr(self) -> NonNull<L> {
        NonNull::from_ref(self)
    }

    fn from_ptr(ptr: NonNull<L>) -> Self {
        unsafe { ptr.as_ref() }
    }
}

pub struct BumpyBook<R: RefLike<L>, L: Ledger> {
    #[cfg(test)]
    total_allocations: usize,
    #[cfg(test)]
    expansions: usize,
    alo: Option<Bump<8>>,
    free: Vec<R>,
    _phantom: PhantomData<L>,
}

impl<R: RefLike<L>, L: Ledger + 'static> BumpyBook<R, L> {
    pub(crate) const fn new() -> Self {
        Self {
            #[cfg(test)]
            expansions: 0,
            #[cfg(test)]
            total_allocations: 0,
            alo: None,
            free: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<R: RefLike<L>, L: Ledger> LedgerBook<L> for BumpyBook<R, L> {
    unsafe fn deallocate(&mut self, ledger: NonNull<L>) {
        let ledger_ref = unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            ledger.as_ref()
        };

        if ledger_ref.reallocation() != NonZeroU64::MAX {
            self.free.push(R::from_ptr(ledger));
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
        self.total_allocations = 0;
        self.free.clear();
    }

    fn allocate<F: FnMut() -> L>(&mut self, mut produce: F) -> NonNull<L> {
        #[cfg(test)]
        {
            self.total_allocations += 1;
        }
        let res = self.free.pop();
        if let Some(x) = res {
            return x.to_ptr();
        } else {
            let alo = if let Some(ref mut alo) = self.alo {
                alo
            } else {
                self.alo = Some(Bump::with_min_align());
                self.alo.as_mut().unwrap()
            };

            NonNull::from_mut(alo.alloc(produce()))
        }
    }
}
