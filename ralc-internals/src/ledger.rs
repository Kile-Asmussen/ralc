use core::alloc;
use std::{
    alloc::{Allocator, Layout},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::accounts::{AccPtr, Account};

trait LedgerAllocator<const CHUNK_SIZE: usize> {
    type Account<'a>: Account
    where
        Self: 'a;

    type AccountRef<'a>: Deref<Target = Self::Account<'a>>
    where
        Self: 'a;

    type ChunkRef<'a>: DerefMut + Deref<Target = FreeListChunk<Self::Account<'a>, CHUNK_SIZE>>
    where
        Self: 'a;

    fn allocate_chunk(&self) -> Self::ChunkRef<'_>;
    fn allocate_account(&self) -> Self::AccountRef<'_>;
}

struct Ledger<'a, L: LedgerAllocator<CHUNK_SIZE> + 'a, const CHUNK_SIZE: usize> {
    current: FreeListPtr<L::Account<'a>, CHUNK_SIZE>,
    allocator: L,
}

impl<'a, A: Account, L: LedgerAllocator<CHUNK_SIZE>, const CHUNK_SIZE: usize>
    Ledger<'a, L, CHUNK_SIZE>
{
    fn new(allocator: L) -> Self {
        let current = FreeListPtr {
            ptr: NonNull::from_ref(&*allocator.allocate_chunk()),
        };
        Self { current, allocator }
    }
}

#[repr(C)]
struct FreeListHeader<A: Account, const CHUNK_SIZE: usize> {
    len: usize,
    prev: Option<FreeListPtr<A, CHUNK_SIZE>>,
    next: Option<FreeListPtr<A, CHUNK_SIZE>>,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct FreeListPtr<A: Account, const CHUNK_SIZE: usize> {
    ptr: NonNull<FreeListChunk<A, CHUNK_SIZE>>,
}

#[repr(C)]
struct FreeListChunk<A: Account, const CHUNK_SIZE: usize> {
    header: FreeListHeader<A, CHUNK_SIZE>,
    data: [Option<AccPtr<A>>; CHUNK_SIZE],
}
