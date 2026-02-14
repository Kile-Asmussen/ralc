use super::*;

#[test]
fn predicatble_allocation_count_pool() {
    let pool = PoolAllocator::new_local();
    allocation_count(
        |i| pool.ralc(i),
        || pool.total_allocations(),
        || pool.free_count(),
    );
}

#[test]
fn test_pool_write_read() {
    let pool = PoolAllocator::new_local();
    test_write_read(pool.ralc(0));
}

#[test]
fn test_pool_into_inner() {
    let pool = PoolAllocator::new_local();
    test_into_inner(pool.ralc(0));
}
