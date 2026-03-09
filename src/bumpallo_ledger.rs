use std::{
    mem::zeroed,
    ops::Deref,
    ptr::{NonNull, null_mut},
};

use bumpalo::Bump;
use ralc_internals::accounts::{AccPtr, Account};

struct Ledger<A: Account + Default> {
    arena: Bump<8>,
    freelist: FreeList<A>,
}

impl<A: Account + Default> Ledger<A> {}

struct FreeList<A: Account> {
    latest: Option<NonNull<FreeChunk<A>>>,
    len: usize,
}

#[repr(C)]
struct FreeChunkHeader<A: Account> {
    prev: Option<NonNull<FreeChunk<A>>>,
    next: Option<NonNull<FreeChunk<A>>>,
    len: usize,
    cap: usize,
}

#[repr(C)]
struct FreeChunk<A: Account, AA: ?Sized + AsRef<[Option<AccPtr<A>>]> = [Option<AccPtr<A>>]> {
    header: FreeChunkHeader<A>,
    data: AA,
}

impl<A: Account> FreeChunk<A> {
    fn new(cap: usize) -> NonNull<Self> {
        let full_cap = cap + size_of::<FreeChunkHeader<A>>();

        let alloc_box = vec![0usize; full_cap].into_boxed_slice();
        let alloc = NonNull::from_ref(&alloc_box);
        std::mem::forget(alloc_box);

        let mut cast = alloc.cast::<FreeChunk<A, [Option<AccPtr<A>>; 0]>>();
        unsafe {
            cast.as_mut().header.cap = cap;
        }
    }
}
