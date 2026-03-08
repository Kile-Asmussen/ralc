pub mod account;

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
