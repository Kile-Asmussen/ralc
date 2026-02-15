use std::time::Instant;
#[cfg(feature = "bumpalo")]
use std::{cell::RefCell, ops::DerefMut, time::Instant};

#[cfg(feature = "bumpalo")]
use parking_lot::Mutex;

#[cfg(not(feature = "bumpalo"))]
use crate::allocators::GlobalAllocator;
#[cfg(feature = "bumpalo")]
use crate::allocators::GlobalAllocator;
#[cfg(feature = "bumpalo")]
use crate::ledgers::{silo::SiloedLedger, sync::SyncLedger};
use crate::{
    OwnedRalc,
    allocators::{AllocatedLedger, LedgerAllocator, PoolAllocator, ThreadLocalAllocator},
    test::MUTEX,
};

const N: usize = 10_000_000;

#[test]
#[cfg(feature = "bumpalo")]

fn stress_test_2_global_bumpalo() {
    let _lock = MUTEX.lock();
    #[cfg(feature = "bumpalo")]
    stress_test_2::<GlobalAllocator>();
    #[cfg(not(feature = "bumpalo"))]
    stress_test_2::<OtherGlobalAllocator>();
}

#[test]
fn stress_test_2_global_non_bumpalo() {
    let _lock = MUTEX.lock();
    #[cfg(not(feature = "bumpalo"))]
    stress_test_2::<GlobalAllocator>();
    #[cfg(feature = "bumpalo")]
    stress_test_2::<OtherGlobalAllocator>();
}

#[test]
#[cfg(feature = "bumpalo")]

fn stress_test_2_threadlocal_bumpalo() {
    let _lock = MUTEX.lock();
    #[cfg(feature = "bumpalo")]
    stress_test_2::<ThreadLocalAllocator>();
    #[cfg(not(feature = "bumpalo"))]
    stress_test_2::<OtherThreadLocalAllocator>();
}

#[test]
fn stress_test_2_threadlocal_non_bumpalo() {
    let _lock = MUTEX.lock();
    #[cfg(not(feature = "bumpalo"))]
    stress_test_2::<ThreadLocalAllocator>();
    #[cfg(feature = "bumpalo")]
    stress_test_2::<OtherThreadLocalAllocator>();
}

#[test]
fn stress_test2_pool() {
    let pool = PoolAllocator::new_local();
    pool.set_chunks(2048, 2048);
    let mut vec = Vec::with_capacity(N);
    let mut vec2 = Vec::with_capacity(N);
    for i in 0..N {
        vec.push(Box::new(i));
    }
    for j in 1..=10 {
        let now = Instant::now();
        for b in vec.drain(..) {
            vec2.push(pool.ralc_box(b));
        }

        let now2 = Instant::now();
        for b in vec2.drain(..) {
            vec.push(b.try_into_box().unwrap())
        }
        println!(
            "Iteration {j} box-ralc took {}, ralc-box took {}",
            now.elapsed().as_secs_f64(),
            now2.elapsed().as_secs_f64()
        );

        vec.clear();
    }
    println!("Total expansions: {}", pool.expansions());
    println!("Total allocations: {}", pool.total_allocations());
}

fn stress_test_2<A: LedgerAllocator>() {
    A::set_chunks(2048, 2048);
    A::reset();
    let mut vec = Vec::with_capacity(N);
    let mut vec2 = Vec::with_capacity(N);
    for i in 0..N {
        vec.push(Box::new(i));
    }
    for j in 1..=10 {
        let now = Instant::now();
        for b in vec.drain(..) {
            vec2.push(OwnedRalc::<_, AllocatedLedger<A>>::from(b));
        }

        let now2 = Instant::now();
        for b in vec2.drain(..) {
            vec.push(b.try_into_box().unwrap())
        }
        println!(
            "Iteration {j} box-ralc took {}, ralc-box took {}",
            now.elapsed().as_secs_f64(),
            now2.elapsed().as_secs_f64()
        );

        vec.clear();
    }
    println!("Total expansions: {}", A::expansions());
    println!("Total allocations: {}", A::total_allocations());
}

//////////////////////////

#[cfg(feature = "bumpalo")]
thread_local! {
    static RALC: RefCell<RetainingBook<SiloedLedger>> = RefCell::new(RetainingBook::new());
}

#[cfg(feature = "bumpalo")]
pub struct OtherThreadLocalAllocator;

#[cfg(feature = "bumpalo")]

impl LedgerAllocator for OtherThreadLocalAllocator {
    type WrappedLedger = SiloedLedger;
    type Allocator = RetainingBook<SiloedLedger>;

    const LIFETIME_NAME: &'static str = "'thread";

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X {
        RALC.with(|ralc| scope(ralc.borrow_mut().deref_mut()))
    }
}

////////////

#[cfg(feature = "bumpalo")]

pub struct OtherGlobalAllocator;

#[cfg(feature = "bumpalo")]

impl LedgerAllocator for OtherGlobalAllocator {
    type WrappedLedger = SyncLedger;
    type Allocator = LeakyBook<SyncLedger>;

    fn with<X, F: FnOnce(&mut Self::Allocator) -> X>(scope: F) -> X {
        static RALC: Mutex<LeakyBook<SyncLedger>> = Mutex::new(LeakyBook::new());
        scope(&mut RALC.lock())
    }

    const LIFETIME_NAME: &'static str = "'static";
}
