use core::fmt;
use std::{cell::Cell, num::NonZeroU32, u64};

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
pub unsafe trait Permits {
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

    /// Convenience method that waits in blocking fashion
    /// availablility of a reference permit and acquire one.
    fn reference_permit(&self) -> bool;

    /// Convenience method that waits in blocking fashion
    /// for availablility of the mutation permit and acquire it.
    fn mutation_permit(&self) -> bool;

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

macro_rules! impl_permit_for_cell {
    ($int:ty) => {
        unsafe impl Permits for Cell<$int> {
            #[inline]
            fn try_reference(&self) -> bool {
                if self.get() < <$int>::MAX {
                    self.update(|n| n + 1);
                    true
                } else {
                    false
                }
            }

            #[inline]
            fn try_mutation(&self) -> bool {
                if self.get() == 0 {
                    self.set(<$int>::MAX);
                    true
                } else {
                    false
                }
            }

            #[inline]
            fn reference_permit(&self) -> bool {
                self.try_reference()
            }

            #[inline]
            fn mutation_permit(&self) -> bool {
                self.try_mutation()
            }

            #[inline]
            unsafe fn try_escalate(&self) -> bool {
                if self.get() == 1 {
                    self.set(<$int>::MAX);
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
    };
}

impl_permit_for_cell!(u64);
impl_permit_for_cell!(u32);
