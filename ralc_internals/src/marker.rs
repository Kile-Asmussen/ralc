/// # Safety requirements
/// 1. Implementor must be `#[repr(transparent(u8)]`.
/// 2. Must only have one variant
pub unsafe trait Marker: Copy + Default {}

#[macro_export]
macro_rules! declare_marker_type {
    ($name:ident, $val:literal) => {
        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
        #[repr(u8)]
        enum $name {
            #[default]
            $name = $val,
        }
        unsafe impl Marker for $name {}
    };
}

/// 56 bit partial integer used for storing reallocation
/// counts inside the ralc pointer struct while allowing
/// space for a [`Marker`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct U56([u8; 7]);

impl U56 {
    /// Convert this `U56` to `u64` for general use.
    pub fn to_u64(self) -> u64 {
        let s = self.0;
        #[cfg(target_endian = "big")]
        let bytes = [0, s[0], s[1], s[2], s[3], s[4], s[5], s[6]];
        #[cfg(target_endian = "little")]
        let bytes = [s[0], s[1], s[2], s[3], s[4], s[5], s[6], 0];
        return u64::from_ne_bytes(bytes);
    }

    /// Create a `U56` from the lowest 56 bits of an `u64`.
    pub fn from_u64(n: u64) -> U56 {
        let b = n.to_ne_bytes();
        #[cfg(target_endian = "little")]
        return U56([b[0], b[1], b[2], b[3], b[4], b[5], b[6]]);
        #[cfg(target_endian = "big")]
        return U56([b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
    }
}
