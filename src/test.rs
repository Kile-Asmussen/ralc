use std::num::NonZeroU64;

use crate::{OwnedRalc, cookie::CookieJar, ledgers::Ledger};

#[test]
fn test_global_into_inner() {
    let owned = OwnedRalc::new_global(0i32);
    let ledger = owned.global_ledger();
    let reallocation = ledger.reallocation();
    let get_reallocation = || ledger.reallocation();

    test_into_inner(owned, get_reallocation, Some(reallocation));
}

#[test]
fn test_global_write_read() {
    let owned = OwnedRalc::new_global(0i32);
    let ledger = owned.global_ledger();
    let reallocation = ledger.reallocation();
    let get_reallocation = || ledger.reallocation();

    test_write_read(owned, get_reallocation, Some(reallocation));
}

#[cfg(test)]
fn test_into_inner(
    owned: OwnedRalc<i32, impl Ledger>,
    mut get_reallocation: impl FnMut() -> NonZeroU64,
    reallocation: Option<NonZeroU64>,
) {
    assert_eq!(reallocation.map(|_| get_reallocation()), reallocation);

    let mut wr = owned.write();
    *wr = 1;
    assert_eq!(owned.ledger().cookie().count(), u32::MAX);
    std::mem::drop(wr);

    assert_eq!(owned.ledger().cookie().count(), 0);

    let res = owned.try_into_inner().unwrap();

    assert_ne!(reallocation, Some(get_reallocation()));

    assert_eq!(res, 1)
}

#[cfg(test)]
fn test_write_read(
    owned: OwnedRalc<i32, impl Ledger>,
    mut get_reallocation: impl FnMut() -> NonZeroU64,
    reallocation: Option<NonZeroU64>,
) {
    let mut wr = owned.write();
    *wr += 1;
    assert_eq!(owned.ledger().cookie().count(), u32::MAX);
    std::mem::drop(wr);

    assert_eq!(reallocation.map(|_| get_reallocation()), reallocation);

    assert_eq!(owned.ledger().cookie().count(), 0);

    let rd = owned.read();
    assert_eq!(owned.ledger().cookie().count(), 1);
    let res = *rd;
    std::mem::drop(rd);

    assert_eq!(reallocation.map(|_| get_reallocation()), reallocation);

    assert_eq!(owned.ledger().cookie().count(), 0);

    assert_eq!(res, 1);

    std::mem::drop(owned);
    assert_eq!(Some(get_reallocation()), reallocation);
}
