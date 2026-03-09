use ralc_internals::{RalcRaw, accounts::Account, declare_marker_type, marker::Marker};

mod bumpallo_ledger;
mod global;

declare_marker_type!(Boxed, 1);
declare_marker_type!(Mutable, 2);
declare_marker_type!(Reference, 3);
declare_marker_type!(Pointer, 4);

#[repr(transparent)]
pub struct RalcBox<T, A: Account>(RalcRaw<A, Boxed, T>);

impl<T, A: Account> Drop for RalcBox<T, A> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY:
            // Invariant
            self.0.drop_box();
        }
    }
}

#[repr(transparent)]
pub struct RalcMut<T, A: Account>(RalcRaw<A, Mutable, T>);

impl<T, A: Account> Drop for RalcMut<T, A> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY:
            // Invariant
            self.0.drop_mut();
        }
    }
}

#[repr(transparent)]
pub struct RalcRef<T, A: Account>(RalcRaw<A, Reference, T>);

impl<T, A: Account> Drop for RalcRef<T, A> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY:
            // Invariant
            self.0.drop_ref();
        }
    }
}

impl<T, A: Account> Clone for RalcRef<T, A> {
    fn clone(&self) -> Self {
        unsafe {
            // SAFETY:
            // Invariant
            self.0.clone_ref();
        }
        Self(self.0)
    }
}

#[repr(transparent)]
pub struct RalcPtr<T, A: Account>(RalcRaw<A, Pointer, T>);
