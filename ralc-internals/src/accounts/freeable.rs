use crate::accounts::permits::Permits;

/// # Safety requirements
/// 1. `free` must release a held mutation permit.
pub unsafe trait Freeable: Permits {
    /// # Safety
    /// 1. After calling it, no other interactions may be made with this object.
    /// 2. There must be a`mut` permit is invoked on [`.permits()`](Self::permits)
    unsafe fn free(&self) {
        unsafe {
            // SAFETY:
            // Guaranteed by caller.
            self.abandon_mutation();
        }
        // IMPL SAFETY:
        // 1. See above
    }
}
