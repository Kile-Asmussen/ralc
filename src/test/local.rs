use crate::allocators::{ThreadLocal, ThreadLocalAllocator};

use super::*;

#[test]
fn predicatble_allocation_count() {
    predictable_allocation_count_for::<ThreadLocalAllocator>();
}

#[test]
fn borrows_dont_allocate() {
    borrows_dont_allocate_for::<ThreadLocalAllocator>();
}

#[test]
fn test_thread_local_write_read() {
    test_write_read(OwnedRalc::<_, ThreadLocal>::new(0));
}

#[test]
fn test_thread_local_into_inner() {
    test_into_inner(OwnedRalc::<_, ThreadLocal>::new(0));
}
