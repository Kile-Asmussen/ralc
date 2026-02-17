use std::ptr::NonNull;

use crate::{
    accounts::{AccPtr, Account},
    variants::markers::{Boxed, Marker, Mutable, Pointer, Reference, U56, Variant},
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

pub struct RalcVariant<A: Account, T>(RalcRaw<A, Variant, T>);

impl<A: Account, T> Drop for RalcVariant<A, T> {
    fn drop(&mut self) {
        match self.0.variant {
            Variant::Mutable => {
                RalcMutable(RalcRaw {
                    variant: Mutable::Mutable,
                    count: self.0.count,
                    account: self.0.account,
                    data: self.0.data,
                });
            }
            Variant::Boxed => {
                RalcBoxed(RalcRaw {
                    variant: Boxed::Boxed,
                    count: self.0.count,
                    account: self.0.account,
                    data: self.0.data,
                });
            }
            Variant::Reference => {
                RalcReference(RalcRaw {
                    variant: Reference::Reference,
                    count: self.0.count,
                    account: self.0.account,
                    data: self.0.data,
                });
            }
            Variant::Pointer => {}
        }
    }
}

pub struct RalcBoxed<A: Account, T>(RalcRaw<A, Boxed, T>);

impl<A: Account, T> RalcBoxed<A, T> {}

pub struct RalcReference<A: Account, T>(RalcRaw<A, Reference, T>);
pub struct RalcMutable<A: Account, T>(RalcRaw<A, Mutable, T>);

#[derive(Clone, Copy)]
pub struct RalcPointer<A: Account, T>(RalcRaw<A, Pointer, T>);

impl<A: Account, T> Clone for RalcReference<A, T> {
    fn clone(&self) -> Self {
        if self.0.account.try_ref() {
            RalcReference(self.0.clone())
        } else {
            panic!("Invariant violation")
        }
    }
}

impl<A: Account, T> Drop for RalcBoxed<A, T> {
    fn drop(&mut self) {
        self.0.account.invalidate();
        if self.0.account.try_mut() {
            unsafe {
                Box::from_raw(self.0.data.as_ptr());
                self.0.account.free();
            }
        }
    }
}

impl<A: Account, T> Drop for RalcReference<A, T> {
    fn drop(&mut self) {
        if self.0.account.check() != self.0.count.to_u64() {
            unsafe {
                if self.0.account.try_ref_to_mut() {
                    Box::from_raw(self.0.data.as_ptr());
                    self.0.account.free();
                }
            }
        }
    }
}

impl<A: Account, T> Drop for RalcMutable<A, T> {
    fn drop(&mut self) {
        if self.0.account.check() != self.0.count.to_u64() {
            unsafe {
                Box::from_raw(self.0.data.as_ptr());
                self.0.account.free();
            }
        }
    }
}
