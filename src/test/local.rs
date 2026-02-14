use crate::allocators::ThreadLocalAllocator;

use super::*;

#[test]
fn predicatble_allocation_count_local() {
    allocation_count(
        OwnedRalc::new_thread_local,
        ThreadLocalAllocator::reset,
        ThreadLocalAllocator::total_allocations,
        ThreadLocalAllocator::free_count,
    );
}

#[test]
fn test_thread_local_write_read() {
    test_write_read(OwnedRalc::new_thread_local(0));
}

#[test]
fn test_thread_local_into_inner() {
    test_into_inner(OwnedRalc::new_thread_local(0));
}
