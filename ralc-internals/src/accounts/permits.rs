use core::fmt;
use std::{
    cell::Cell,
    num::NonZeroU32,
    sync::atomic::{AtomicU32, Ordering},
    u64,
};

/// A container for permits to access data guarded by
/// this account. This trait can be aither implemented in
/// a shareable fashion using `Sync` data, or in a thread-local
/// fashion.
///
/// Permits come in two kinds:
///
/// - Reference permits allow read-only access.
/// - Mutation permits allow read-write access.
///
/// The two kinds are mutually exclusive, reference permits can
/// be stacked, mutation permits cannot.
///
/// This is the same semantics as any given `RwLock` implementation
/// and deliberately styled after the `parking_lock` crate's `RawRwLock`.
///
/// # Safety
///
/// This trait must obey the semantics of an `RwLock` or `RefCell`.
pub unsafe trait Permits: Sized {
    /// The type of an underlying locking mechanism, if any.
    type UnderlyingLockableEntity;

    /// Acess the implementation-specific locking mechanism, if any.
    ///
    /// # Safety
    ///
    /// Whichever invariants the permits of this type reflects, manipulation of the
    /// locking entity must be taken to alter these invariants.
    unsafe fn underlying(&self) -> &Self::UnderlyingLockableEntity;

    /// Check for availablility of a reference permit and acquire one.
    fn try_reference(&self) -> bool;

    /// Check for availablility of the mutation permit and acquire it.
    fn try_mutation(&self) -> bool;

    /// Attempt to upgrade a reference permit to the mutation permit, i.e. this will
    /// only succeed if there is only a single reference permit.
    ///
    /// # Safety
    /// 1. A reference permit must have been acquired.
    unsafe fn try_escalate(&self) -> bool;

    /// Downgrade existing mutation permit to reference permit.
    ///
    /// # Safety
    /// 1. The mutation permit must have been acquired.
    unsafe fn relax_permit(&self);

    /// Relinquish a reference permit.
    ///
    /// # Safety requirements:
    /// 1. A reference permit must have been acquired.
    unsafe fn abandon_reference(&self);

    /// Relinquish the mutation permit.
    ///
    /// # Safety requirements:
    /// 1. The mutation permit must have been acquired.
    unsafe fn abandon_mutation(&self);
}

unsafe impl Permits for Cell<u32> {
    type UnderlyingLockableEntity = ();

    #[inline]
    unsafe fn underlying(&self) -> &Self::UnderlyingLockableEntity {
        &()
    }

    #[inline]
    fn try_reference(&self) -> bool {
        if self.get() < u32::MAX {
            self.update(|n| n + 1);
            true
        } else {
            false
        }
    }

    #[inline]
    fn try_mutation(&self) -> bool {
        if self.get() == 0 {
            self.set(u32::MAX);
            true
        } else {
            false
        }
    }

    #[inline]
    unsafe fn try_escalate(&self) -> bool {
        if self.get() == 1 {
            self.set(u32::MAX);
            true
        } else {
            false
        }
    }

    #[inline]
    unsafe fn relax_permit(&self) {
        self.set(1)
    }

    #[inline]
    unsafe fn abandon_reference(&self) {
        self.update(|n| n - 1)
    }

    #[inline]
    unsafe fn abandon_mutation(&self) {
        self.set(0)
    }
}

unsafe impl Permits for AtomicU32 {
    type UnderlyingLockableEntity = ();

    unsafe fn underlying(&self) -> &Self::UnderlyingLockableEntity {
        &()
    }

    fn try_reference(&self) -> bool {
        self.fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
            .is_ok()
    }

    fn try_mutation(&self) -> bool {
        self.compare_exchange(0, u32::MAX, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    unsafe fn try_escalate(&self) -> bool {
        self.compare_exchange(1, u32::MAX, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    unsafe fn relax_permit(&self) {
        self.store(1, Ordering::Release);
    }

    unsafe fn abandon_reference(&self) {
        self.fetch_sub(1, Ordering::Release);
    }

    unsafe fn abandon_mutation(&self) {
        self.store(0, Ordering::Release);
    }
}
