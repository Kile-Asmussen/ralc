use std::{
    array, cell::RefCell, collections::VecDeque, mem::ManuallyDrop, num::NonZeroU64, ptr::NonNull,
};

use crate::{OwnedRalc, ledgers::SiloedLedger};

use super::{
    Ledger,
    allocator::{AllocatedLedger, LedgerAllocator},
};

thread_local! {
    static ALLOC: RefCell<Vec<Vec<LocalLedger>>> = RefCell::new(Vec::new());
    static FREE: RefCell<VecDeque<NonNull<LocalLedger>>> = RefCell::new(VecDeque::new());
}

pub struct LocalAllocator;

impl LedgerAllocator for LocalAllocator {
    type WrappedLedger = SiloedLedger;

    fn alloc() -> NonNull<LocalLedger> {
        return FREE.with(|free| {
            let mut free = free.borrow_mut();
            if free.len() == 0 {
                free.append(&mut new_chunk());
            }
            free.pop_front().unwrap()
        });

        fn new_chunk() -> VecDeque<NonNull<LocalLedger>> {
            ALLOC.with(|alloc| {
                let mut alloc = alloc.borrow_mut();
                alloc.push(Vec::from(array::from_fn::<_, 1024, _>(|_| {
                    // SAFETY:
                    // 1. Guaranteed by scope
                    unsafe { LocalLedger::new() }
                })));
                alloc
                    .last()
                    .unwrap()
                    .iter()
                    .map(|x| NonNull::from_ref(x))
                    .collect()
            })
        }
    }

    unsafe fn free(ledger: NonNull<LocalLedger>) {
        let ledger_ref = unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            ledger.as_ref()
        };
        if ledger_ref.generation() != NonZeroU64::MAX {
            FREE.with(|free| {
                let mut free = free.borrow_mut();
                free.push_back(ledger);
            })
        }
    }
}

pub type LocalLedger = AllocatedLedger<LocalAllocator>;

impl<T> OwnedRalc<T, LocalLedger> {
    pub fn new_local(data: T) -> Self {
        let data = Box::new(ManuallyDrop::new(data));
        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Self-evident
            Self::from_parts(
                LocalAllocator::alloc(),
                // SAFETY:
                // 1. Guaranteed by Box
                NonNull::new_unchecked(Box::into_raw(data)),
            )
        }
    }
}
