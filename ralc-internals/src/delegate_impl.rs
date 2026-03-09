use crate::accounts::{balances::Balance, permits::Permits};

/// A simple combination of a separate [`Balance`](accounts::Balance) and [`Permits`](accounts::Permits)
/// intended to be wrapped in a newtype with an appropriate `Freeable` implentation which interacts with
/// the [`SimpleAccount::data`] field.
pub trait DelegateAccountImpl {
    type DelegatedBalance: Balance;
    type DelegatedPermits: Permits;
    fn balance(&self) -> &Self::DelegatedBalance;
    fn permits(&self) -> &Self::DelegatedPermits;
}

#[macro_export]
macro_rules! delegate_account_impl {
    ($delegator:ty) => {
        trait CheckDelegateAccountImpl: DelegateAccountImpl {}
        impl CheckDelegateAccountImpl for $delegator {}

        // SAFETY:
        // 1. delegated implementation.
        unsafe impl Balance for $delegator {
            fn invalidate(&self) {
                self.balance().invalidate();
            }

            fn check(&self) -> u64 {
                self.balance().check()
            }
        }

        // SAFETY:
        // 1. delegated implementation.
        unsafe impl<B: Balance, P: Permits> Permits for $delegator {
            fn try_reference(&self) -> bool {
                self.permits().try_reference()
            }

            fn try_mutation(&self) -> bool {
                self.permits().try_mutation()
            }

            unsafe fn try_escalate(&self) -> bool {
                unsafe {
                    // SAFETY:
                    // 1. Guaranteed by caller
                    self.permits().try_escalate()
                }
            }

            unsafe fn relax_permit(&self) {
                unsafe {
                    // SAFETY:
                    // 1. Guaranteed by caller
                    self.permits().relax_permit()
                }
            }

            unsafe fn abandon_reference(&self) {
                unsafe {
                    // SAFETY:
                    // 1. Guaranteed by caller
                    self.permits().abandon_reference()
                }
            }

            unsafe fn abandon_mutation(&self) {
                unsafe {
                    // SAFETY:
                    // 1. Guaranteed by caller
                    self.permits().abandon_mutation()
                }
            }

            type UnderlyingLockableEntity = P::UnderlyingLockableEntity;

            unsafe fn underlying(&self) -> &Self::UnderlyingLockableEntity {
                unsafe {
                    // SAFETY:
                    // Delegated responsibility
                    self.permits().underlying()
                }
            }
        }
    };
}
