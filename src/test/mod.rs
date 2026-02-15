use std::num::NonZeroU64;

use parking_lot::Mutex;

use crate::{
    OwnedRalc,
    allocators::{AllocatedLedger, GlobalAllocator, LedgerAllocator, PoolAllocator},
    cookie::CookieJar,
    ledgers::Ledger,
};

mod global;
mod local;
mod pool;

fn predictable_allocation_count_for<A: LedgerAllocator>() {
    predictable_allocation_count(
        OwnedRalc::<_, AllocatedLedger<A>>::new,
        A::reset,
        A::total_allocations,
        A::free_count,
    );
}

fn predictable_allocation_count<L: Ledger>(
    new: impl Fn(i32) -> OwnedRalc<i32, L>,
    reset: impl Fn(),
    total_allocations: impl Fn() -> usize,
    free_count: impl Fn() -> usize,
) {
    reset();
    let mut vec = vec![];
    for i in 0..1000 {
        vec.push(new(i))
    }
    assert_eq!(1000, total_allocations());
    for or in vec {
        test_write_read(or);
    }
    assert!(free_count() >= 1000);
}

fn borrows_dont_allocate_for<A: LedgerAllocator>() {
    borrows_dont_allocate(
        OwnedRalc::<_, AllocatedLedger<A>>::new,
        A::reset,
        A::total_allocations,
    );
}

fn borrows_dont_allocate<L: Ledger>(
    new: impl Fn(i32) -> OwnedRalc<i32, L>,
    reset: impl Fn(),
    total_allocations: impl Fn() -> usize,
) {
    reset();
    let owned = new(0);
    let mut vec = vec![];
    for _ in 0..1000 {
        vec.push(owned.borrow());
    }
    assert_eq!(1, total_allocations());
    for or in vec {
        *or.write().unwrap() += 1;
    }
    assert_eq!(*owned.read().unwrap(), 1000);
}

fn test_into_inner(owned: OwnedRalc<i32, impl Ledger>) {
    test_into_inner_full(owned, || NonZeroU64::new(1).unwrap(), None);
}

fn test_write_read(owned: OwnedRalc<i32, impl Ledger>) {
    test_write_read_full(owned, || NonZeroU64::new(1).unwrap(), None);
}

fn test_into_inner_full(
    owned: OwnedRalc<i32, impl Ledger>,
    mut get_reallocation: impl FnMut() -> NonZeroU64,
    reallocation: Option<NonZeroU64>,
) {
    assert_eq!(reallocation.map(|_| get_reallocation()), reallocation);

    let mut wr = owned.write().unwrap();
    *wr = 99;
    assert_eq!(owned.ledger().cookie().count(), u32::MAX);
    std::mem::drop(wr);

    assert_eq!(owned.ledger().cookie().count(), 0);

    let res = *owned.try_into_box().unwrap();

    assert_ne!(reallocation, Some(get_reallocation()));

    assert_eq!(res, 99)
}

fn test_write_read_full(
    owned: OwnedRalc<i32, impl Ledger>,
    mut get_reallocation: impl FnMut() -> NonZeroU64,
    reallocation: Option<NonZeroU64>,
) {
    let mut wr = owned.write().unwrap();
    *wr = 99;
    assert_eq!(owned.ledger().cookie().count(), u32::MAX);
    std::mem::drop(wr);

    assert_eq!(reallocation.map(|_| get_reallocation()), reallocation);

    assert_eq!(owned.ledger().cookie().count(), 0);

    let rd = owned.read().unwrap();
    assert_eq!(owned.ledger().cookie().count(), 1);
    let res = *rd;
    std::mem::drop(rd);

    assert_eq!(reallocation.map(|_| get_reallocation()), reallocation);

    assert_eq!(owned.ledger().cookie().count(), 0);

    assert_eq!(res, 99);

    std::mem::drop(owned);
    assert_ne!(reallocation, Some(get_reallocation()));
}
