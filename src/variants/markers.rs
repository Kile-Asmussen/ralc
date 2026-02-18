use crate::variants::markers::secret::Sealed;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Mutable {
    #[default]
    Mutable = 1,
}
impl Marker for Mutable {}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Boxed {
    #[default]
    Boxed = 2,
}
impl Marker for Boxed {}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Reference {
    #[default]
    Reference = 3,
}
impl Marker for Reference {}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Pointer {
    #[default]
    Pointer = 4,
}
impl Marker for Pointer {}

pub trait Marker: Default + std::fmt::Debug + PartialEq + Eq + Clone + Copy + Sealed {}

mod secret {
    pub trait Sealed {}
    impl Sealed for super::Pointer {}
    impl Sealed for super::Reference {}
    impl Sealed for super::Boxed {}
    impl Sealed for super::Mutable {}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct U56([u8; 7]);

impl U56 {
    pub fn to_u64(self) -> u64 {
        let s = self.0;
        #[cfg(target_endian = "big")]
        let bytes = [0, s[0], s[1], s[2], s[3], s[4], s[5], s[6]];
        #[cfg(target_endian = "little")]
        let bytes = [s[0], s[1], s[2], s[3], s[4], s[5], s[6], 0];
        return u64::from_ne_bytes(bytes);
    }

    pub fn from_u64(n: u64) -> U56 {
        let b = n.to_ne_bytes();
        #[cfg(target_endian = "little")]
        return U56([b[0], b[1], b[2], b[3], b[4], b[5], b[6]]);
        #[cfg(target_endian = "big")]
        return U56([b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
    }
}
