use std::{num::NonZeroU64, ptr::NonNull};

use crate::ledgers::Ledger;
#[cfg(feature = "bumpalo")]
mod bumpalo;
mod leaking;
mod retaining;

#[cfg(feature = "bumpalo")]
pub use bumpalo::BumpyBook;
pub use leaking::LeakyBook;
pub use retaining::RetainingBook;

#[cfg(not(test))]
const CHUNK_SIZE: usize = 1024;

#[cfg(test)]
const CHUNK_SIZE: usize = 16;

pub trait LedgerBook<L: Ledger + Clone> {
    type Storage: RefLike<L>;

    fn free_list(&mut self) -> &mut Vec<Self::Storage>;

    fn lazy_init(&mut self);
    fn new_slice(&mut self, sample: &L) -> (&[L], &mut Vec<Self::Storage>);

    #[cfg(test)]
    fn count_expansion(&mut self);

    #[cfg(test)]
    fn expansions(&self) -> usize;

    #[cfg(test)]
    fn count_allocation(&mut self);

    #[cfg(test)]
    fn total_allocations(&self) -> usize;

    #[cfg(test)]
    fn free_count(&self) -> usize;

    #[cfg(test)]
    fn reset(&mut self);

    fn set_chunks(&mut self, chunk: usize, limit: usize);
}

pub trait LedgerBookExt<L: Ledger + Clone>: LedgerBook<L> {
    /// # Safety requirements
    /// 1. `ledger` is a return from next_free
    /// 2. `free` is not called for the same pointer twice
    #[inline]
    unsafe fn deallocate(&mut self, ledger: NonNull<L>) {
        let ledger_ref = unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            ledger.as_ref()
        };

        if ledger_ref.reallocation() != NonZeroU64::MAX {
            self.free_list().push(RefLike::from_ptr(ledger));
        }
    }

    #[inline]
    fn allocate(&mut self, sample: &L) -> NonNull<L> {
        #[cfg(test)]
        self.count_allocation();

        if let Some(r) = self.free_list().pop() {
            return r.to_ptr();
        }

        #[cfg(test)]
        self.count_expansion();

        self.lazy_init();

        let (slice, free_list) = self.new_slice(sample);

        free_list.extend(
            (&slice[1..])
                .iter()
                .map(|r| Self::Storage::from_ptr(NonNull::from_ref(r))),
        );

        NonNull::from_ref(&slice[0])
    }
}

impl<L: Ledger + Clone, B: LedgerBook<L>> LedgerBookExt<L> for B {}

pub trait RefLike<L>: Copy + Sized {
    fn to_ptr(self) -> NonNull<L>;
    fn from_ptr(ptr: NonNull<L>) -> Self;
}

impl<L> RefLike<L> for NonNull<L> {
    #[inline]
    fn to_ptr(self) -> NonNull<L> {
        self
    }

    #[inline]
    fn from_ptr(ptr: NonNull<L>) -> Self {
        ptr
    }
}

impl<L> RefLike<L> for &'static L {
    #[inline]
    fn to_ptr(self) -> NonNull<L> {
        NonNull::from_ref(self)
    }

    #[inline]
    fn from_ptr(ptr: NonNull<L>) -> Self {
        unsafe { ptr.as_ref() }
    }
}
