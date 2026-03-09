use std::ptr::NonNull;

use crate::{
    accounts::{AccPtr, Account},
    marker::{Marker, U56},
};

pub mod accounts;
pub mod delegate_impl;
pub mod marker;

pub use private::RalcRaw;

mod private {
    use super::*;

    /// The raw Reallocation Counting pointer, for use in implementing libraries. Such a
    /// library should implement a transparent wrapper struct with a suitable [`Drop`] implementation.
    ///
    /// `RalcRaw` implements four different states. These should be distinguished
    /// in library code by using different marker types inside the wrapper types, to allow
    /// niche optimization.
    ///
    /// 1. A `RalcRaw` can be in an "owned" state, in which it holds responsibility for
    ///    tracking the reallocation count and no permits. In this case, the wrapper type's
    ///    `Drop` must call [`RalcRaw::drop_box`].
    /// 2. A `RalcRaw` can be in an "writing" state, in which it holds the mutation permit of its account
    ///    and therefore can act the sole exclusive mutable reference to the allocated data and a.
    ///    In this case, the wrapper type's `Drop` must call [`RalcRaw::drop_mut`].
    /// 3. A `RalcRaw` can be in an "reading" state, in which it holds a reference permit of its account
    ///    and therefore can act as a sharable reference.
    ///    In this case, the wrapper type's `Drop` must call [`RalcRaw::drop_ref`].
    /// 4. A `RalcRaw` can be in an "weak" state, in which it holds no permits and no responsibility.
    ///    In this case, the wrapper type's `Drop` must not call any of the dropping helper methods.
    pub struct RalcRaw<A: Account, V: Marker, T> {
        _variant: V,
        count: U56,
        account: AccPtr<A>,
        /// # Safety invariants:
        /// 1. `data` is always derived from a box
        data: NonNull<T>,
    }

    impl<A: Account, V: Marker, T> RalcRaw<A, V, T> {
        /// Create a new `RalcRaw` in the "owned" state
        ///
        /// # Safety
        /// 1. The account pointer must not be shared with another `RalcRaw` in the "owned" state
        #[inline]
        pub unsafe fn from_parts(ptr: AccPtr<A>, data: Box<T>) -> Self {
            RalcRaw {
                _variant: V::default(),
                count: ptr.check().into(),
                account: ptr,
                data: unsafe { NonNull::new_unchecked(Box::into_raw(data)) },
            }
        }

        /// TODO
        /// Change the marker type
        #[inline]
        pub fn switch_makrer<W: Marker>(self) -> RalcRaw<A, W, T> {
            RalcRaw {
                _variant: W::default(),
                count: self.count,
                account: self.account,
                data: self.data,
            }
        }

        /// TODO
        /// # Safety
        /// 1. This pointer must be in the "owned" state
        #[inline]
        pub unsafe fn disown(self) {
            self.account.invalidate();
        }

        /// TODO
        #[inline]
        pub fn is_disowned(self) -> bool {
            self.account.check() == self.count.into()
        }

        /// TODO
        /// # Safety
        /// 1. Counts as drop
        /// 2. A mutation permit must be held
        #[inline]
        unsafe fn drop_with_mutation(self) {
            let alloc = unsafe {
                // SAFETY:
                // 1. Guaranteed by invariant
                Box::from_raw(self.data.as_ptr())
            };
            std::mem::drop(alloc);

            unsafe {
                // SAFETY:
                // 1. Guaranteed by caller
                // 2. Ensured by call to try_mutation above
                self.account.free();
            }
        }

        /// TODO
        /// # Safety
        /// 1. This must only be called during drop, and counts as having dropped the underlying data
        ///    and tracking account.
        /// 2. This must only be called if `self` is in the "owned" state.
        #[inline]
        pub unsafe fn drop_box(self) {
            unsafe {
                // SAFETY:
                // 1. Guaranteed by caller
                self.disown();
            }

            if self.account.try_mutation() {
                unsafe {
                    // SAFETY:
                    // 1. Guaranteed by caller
                    // 2. try_mutation called above
                    self.drop_with_mutation();
                }
            }
        }

        /// TODO
        /// # Safety
        /// 1. This must only be called during drop, and counts as having dropped the underlying data
        ///    and tracking account.
        /// 2. This must only be called if `self` is in the "reading" state.
        #[inline]
        pub unsafe fn drop_ref(self) {
            if self.is_disowned() {
                return;
            }

            let escalated = unsafe {
                // SAFETY:
                // 1. Guaranteed by "reading" state
                self.account.try_escalate()
            };

            if escalated {
                unsafe {
                    // SAFETY:
                    // 1. Guaranteed by caller
                    // 2. try_escalate called above
                    self.drop_with_mutation();
                }
            }
        }

        /// TODO
        /// # Safety
        /// 1. This must only be called during drop, and counts as having dropped the underlying data
        ///    and tracking account.
        /// 2. This must only be called if `self` is in the "writing" state.
        #[inline]
        pub unsafe fn drop_mut(self) {
            if self.is_disowned() {
                return;
            }

            unsafe {
                // SAFETY:
                // 1. Guaranteed by caller
                // 2. Guaranteed by "writing" state
                self.drop_with_mutation();
            }
        }

        /// If this allocation has been disowned and we are in a "writing" state, we can
        /// reclaim it, transmuting into an "owning" reference.
        ///
        /// If `None` is returned, the original "writing" reference remains valid. Else
        /// it becomes a "weak" reference which is invalid.
        ///
        /// # Safety
        ///
        /// 1. Self must be in a "writing" state and after this call is no longer valid.
        #[inline]
        pub unsafe fn try_reclaim_dropped_box(self) -> Option<Self> {
            if self.is_disowned() {
                let owned = Self {
                    count: self.account.check().into(),
                    ..self
                };
                unsafe {
                    // SAFETY:
                    // 1. Guaranteed by "writing" state
                    self.account.abandon_mutation();
                }
                Some(owned)
            } else {
                None
            }
        }

        /// If this allocation has been disowned and we are in a "writing" state, we can
        /// reclaim it and return an "owning"-sate reference.
        ///
        /// No matter the return, the original "writing" reference remains valid.
        ///
        /// # Safety
        ///
        /// 1. Self must be in a "writing" state
        #[inline]
        pub unsafe fn try_reclaim_dropped_box_retaining_mut(&mut self) -> Option<Self> {
            if self.is_disowned() {
                self.count = self.account.check().into();

                Some(*self)
            } else {
                None
            }
        }

        /// If this allocation has only as single "reading" reference, we can upgrade it into
        /// a "writing" state ference.
        ///
        /// If `None` is returned, the original reference is still in a valid "reading" state.
        /// If `Some` is returned, the original reference is now "weak".
        ///
        /// # Safety
        ///
        /// 1. Self must be in a "reading" state
        #[inline]
        pub unsafe fn try_upgrade_ref_into_mut(self) -> Option<Self> {
            let escalated = unsafe {
                // SAFETY:
                // 1. Guaranteed by reading state
                self.account.try_escalate()
            };
            if escalated { Some(self) } else { None }
        }

        /// TODO
        /// # Safety
        ///
        /// 1. Self must be in a "writing" state
        #[inline]
        pub unsafe fn downgrade_mut_into_ref(self) -> Self {
            unsafe {
                // SAFETY:
                // 1. Guaranteed by caller
                self.account.relax_permit();
            }
            self
        }

        /// TODO
        /// # Safety
        ///
        /// 1. Self must be in a "reading" state
        #[inline]
        pub unsafe fn clone_ref(self) -> Self {
            self.account.try_reference();
            self
        }

        /// TODO
        /// # Safety
        ///
        /// 1. Self must be in an "owned" state
        #[inline]
        pub unsafe fn try_acquire_ref(self) -> Option<Self> {
            if self.account.try_reference() {
                Some(self)
            } else {
                None
            }
        }

        /// TODO
        /// # Safety
        ///
        /// 1. Self must be in an "owned" state
        #[inline]
        pub unsafe fn try_acquire_mut(self) -> Option<Self> {
            if self.account.try_mutation() {
                Some(self)
            } else {
                None
            }
        }

        /// # Safety
        /// TODO
        /// 1. "owned"
        #[inline]
        pub unsafe fn disown_into_mut(self) -> Option<Self> {
            let res = unsafe {
                // SAFETY:
                //
                self.try_acquire_mut()?
            };
            unsafe {
                // SAFETY:
                // 1. Guaranteed by caller
                self.disown()
            };
            Some(res)
        }

        /// # Safety
        /// TODO
        /// 1. "owned"
        #[inline]
        pub unsafe fn disown_into_ref(self) -> Option<Self> {
            let res = unsafe {
                // SAFETY:
                // 1. Guaranteed by caller
                self.try_acquire_mut()?
            };
            unsafe {
                // SAFETY:
                // 1. Guaranteed by caller
                self.disown()
            };
            Some(res)
        }
    }

    impl<A: Account, V: Marker, T> Clone for RalcRaw<A, V, T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<A: Account, V: Marker, T> Copy for RalcRaw<A, V, T> {}
}
