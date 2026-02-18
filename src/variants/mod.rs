use std::ptr::NonNull;

use crate::{
    accounts::{AccPtr, Account},
    variants::markers::{Boxed, Marker, Mutable, Pointer, Reference, U56},
};

pub mod markers;

struct RalcRaw<A: Account, V: Marker, T> {
    variant: V,
    count: U56,
    account: AccPtr<A>,
    data: NonNull<T>,
}

impl<A: Account, V: Marker, T> Clone for RalcRaw<A, V, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Account, V: Marker, T> Copy for RalcRaw<A, V, T> {}

pub struct RalcBoxed<A: Account, T>(RalcRaw<A, Boxed, T>);

impl<A: Account, T> RalcBoxed<A, T> {
    /// # Safety requirements
    /// 1. ONE of the following must hold:
    ///    1. `account` is a reference with `'static` lifetime.
    ///    2. `account` has the lifetime thread locals and `Self: !Send`.
    ///    3. `account` has a shorter lifetime but the type of `A` constrains it.
    /// 2. Account must have no active permits
    unsafe fn from_parts(account: &A, data: Box<T>) -> RalcBoxed<A, T> {
        Self(RalcRaw {
            variant: Boxed::Boxed,
            count: U56::from_u64(account.check()),
            account: unsafe {
                // SAFETY:
                // Guaranteed by caller
                AccPtr::new(account)
            },
            data: unsafe {
                // SAFETY:
                // Box::into_raw always returns non-null
                NonNull::new_unchecked(Box::into_raw(data))
            },
        })
    }
}

#[derive(Clone, Copy)]
pub struct RalcPointer<A: Account, T>(RalcRaw<A, Pointer, T>);

pub struct RalcReference<A: Account, T>(RalcRaw<A, Reference, T>);

impl<A: Account, T> Clone for RalcReference<A, T> {
    fn clone(&self) -> Self {
        if self.0.account.try_ref() {
            RalcReference(self.0.clone())
        } else {
            panic!("Invariant violation")
        }
    }
}

impl<A: Account, T> Drop for RalcReference<A, T> {
    fn drop(&mut self) {
        if self.0.account.check() != self.0.count.to_u64() {
            unsafe {
                if self.0.account.try_ref_to_mut() {
                    std::mem::drop(Box::from_raw(self.0.data.as_ptr()));
                    self.0.account.free();
                }
            }
        }
        unsafe {
            self.0.account.drop_ref();
        }
    }
}

pub struct RalcMutable<A: Account, T>(RalcRaw<A, Mutable, T>);

impl<A: Account, T> Drop for RalcBoxed<A, T> {
    fn drop(&mut self) {
        self.0.account.invalidate();
        if self.0.account.try_mut() {
            unsafe {
                std::mem::drop(Box::from_raw(self.0.data.as_ptr()));
                self.0.account.free();
            }
        }
    }
}

impl<A: Account, T> Drop for RalcMutable<A, T> {
    fn drop(&mut self) {
        if self.0.account.check() != self.0.count.to_u64() {
            unsafe {
                std::mem::drop(Box::from_raw(self.0.data.as_ptr()));
                self.0.account.free();
            }
        }
        unsafe {
            self.0.account.drop_ref();
        }
    }
}
