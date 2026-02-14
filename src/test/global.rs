use super::*;

static MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn predicatble_allocation_count_global() {
    let _lock = MUTEX.lock();
    GlobalAllocator::reset();
    allocation_count(
        OwnedRalc::new_global,
        GlobalAllocator::total_allocations,
        GlobalAllocator::free_count,
    );
}

#[test]
fn test_global_into_inner() {
    let _lock = MUTEX.lock();
    GlobalAllocator::reset();
    let owned = OwnedRalc::new_global(0);
    let ledger = owned.global_ledger();
    let reallocation = ledger.reallocation();
    let get_reallocation = || ledger.reallocation();

    test_into_inner_full(owned, get_reallocation, Some(reallocation));
}

#[test]
fn test_global_write_read() {
    let _lock = MUTEX.lock();
    GlobalAllocator::reset();

    let owned = OwnedRalc::new_global(0i32);
    let ledger = owned.global_ledger();
    let reallocation = ledger.reallocation();
    let get_reallocation = || ledger.reallocation();

    test_write_read_full(owned, get_reallocation, Some(reallocation));
}
