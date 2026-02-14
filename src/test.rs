use crate::{OwnedRalc, cookie::CookieJar, ledgers::Ledger};

#[test]
fn test_into_inner() {
    let owned = OwnedRalc::new_global(0i32);
    let ledger = owned.global_ledger();
    let reallocation = ledger.reallocation();

    let mut wr = owned.write();
    *wr += 1;
    assert_eq!(owned.ledger().cookie().count(), u32::MAX);
    std::mem::drop(wr);
    assert_eq!(ledger.reallocation(), reallocation);

    assert_eq!(owned.ledger().cookie().count(), 0);

    let res = owned.try_into_inner().unwrap();
    assert_ne!(ledger.reallocation(), reallocation);

    assert_eq!(res, 1)
}

#[test]
fn test_write_read() {
    let owned = OwnedRalc::new_global(0i32);
    let ledger = owned.global_ledger();
    let reallocation = ledger.reallocation();

    let mut wr = owned.write();
    *wr += 1;
    assert_eq!(owned.ledger().cookie().count(), u32::MAX);
    std::mem::drop(wr);

    assert_eq!(ledger.reallocation(), reallocation);

    assert_eq!(owned.ledger().cookie().count(), 0);

    let rd = owned.read();
    assert_eq!(owned.ledger().cookie().count(), 1);
    let res = *rd;
    std::mem::drop(rd);

    assert_eq!(ledger.reallocation(), reallocation);

    assert_eq!(owned.ledger().cookie().count(), 0);

    assert_eq!(res, 1);

    std::mem::drop(owned);
    assert_ne!(ledger.reallocation(), reallocation);
}
