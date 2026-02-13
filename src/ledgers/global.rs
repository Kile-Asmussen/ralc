use std::{array, collections::VecDeque, mem::ManuallyDrop, num::NonZeroU64, ptr::NonNull};

use parking_lot::{Mutex, Once};

use crate::{
    OwnedRalc,
    ledgers::{Ledger, SyncLedger},
};

use super::allocator::{AllocatedLedger, LedgerAllocator};

/// # Safety requirements
/// 1. Items in the contained queue must only be used to return as a `NonNull` pointers from alloc
unsafe fn free_list() -> &'static Mutex<VecDeque<&'static GlobalLedger>> {
    static FREE: Mutex<VecDeque<&'static GlobalLedger>> = Mutex::new(VecDeque::new());
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        *FREE.lock() = VecDeque::from_iter(unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            new_chunk()
        });
    });
    return &FREE;
}

/// # Safety requirements
/// 1. The items must only be converted to `NonNull` pointers returned by alloc
unsafe fn new_chunk() -> impl Iterator<Item = &'static GlobalLedger> {
    Vec::from(array::from_fn::<_, 1024, _>(|_| unsafe {
        // SAFETY:
        // 1. Guaranteed by caller
        GlobalLedger::new()
    }))
    .leak()
    .iter()
}

pub struct GlobalAllocator;

impl LedgerAllocator for GlobalAllocator {
    type WrappedLedger = SyncLedger;

    fn alloc() -> NonNull<GlobalLedger> {
        let mut free = unsafe {
            // SAFETY:
            // 1. Self evident
            free_list()
        }
        .lock();
        if free.len() == 0 {
            free.extend(unsafe {
                // SAFETY:
                // 1. Self evident
                new_chunk()
            });
        }
        NonNull::from_ref(free.pop_front().unwrap())
    }

    unsafe fn free(ledger: NonNull<GlobalLedger>) {
        let ledger_ref = unsafe {
            // SAFETY:
            // 1. Guaranteed by caller
            ledger.as_ref()
        };
        if ledger_ref.generation() != NonZeroU64::MAX {
            unsafe {
                // SAFETY:
                // 1. Guaranteed by caller
                free_list().lock().push_back(
                    // SAFETY:
                    // 1. Guaranteed by caller
                    ledger.as_ref(),
                )
            }
        }
    }
}

pub type GlobalLedger = AllocatedLedger<GlobalAllocator>;

impl<T: Send + Sync> OwnedRalc<T, GlobalLedger> {
    pub fn new_global(data: T) -> Self {
        let data = Box::new(ManuallyDrop::new(data));
        unsafe {
            // SAFETY:
            // 1. Guaranteed directly
            // 2. Self-evident
            Self::from_parts(
                GlobalAllocator::alloc(),
                // SAFETY:
                // 1. Guaranteed by Box
                NonNull::new_unchecked(Box::into_raw(data)),
            )
        }
    }

    pub fn global_ledger(&self) -> &'static GlobalLedger {
        unsafe {
            // SAFETY:
            // 1. Guaranteed by GlobalLedger's data existing for the static lifetime
            self.ledger_ptr().as_ref()
        }
    }
}
