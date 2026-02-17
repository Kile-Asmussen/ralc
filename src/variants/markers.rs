#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Variant {
    Mutable = 1,
    Boxed = 2,
    Reference = 3,
    #[default]
    Pointer = 4,
}
impl Marker for Variant {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            1 => Self::Mutable,
            2 => Self::Boxed,
            3 => Self::Reference,
            4 => Self::Pointer,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

//////////////////////

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Dropped {
    Mutable = 1,
    Boxed = 2,
    #[default]
    Reference = 3,
}
unsafe impl IsA<Variant> for Dropped {}
impl Marker for Dropped {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            1 => Self::Mutable,
            2 => Self::Boxed,
            3 => Self::Reference,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Mutexed {
    Mutable = 1,
    Boxed = 2,
    #[default]
    Pointer = 4,
}
unsafe impl IsA<Variant> for Mutexed {}
impl Marker for Mutexed {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            1 => Self::Mutable,
            2 => Self::Boxed,
            4 => Self::Pointer,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Borrowed {
    Mutable = 1,
    Reference = 3,
    #[default]
    Pointer = 4,
}
unsafe impl IsA<Variant> for Borrowed {}
impl Marker for Borrowed {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            1 => Self::Mutable,
            3 => Self::Reference,
            4 => Self::Pointer,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Immutable {
    Boxed = 2,
    Reference = 3,
    #[default]
    Pointer = 4,
}
unsafe impl IsA<Variant> for Immutable {}
impl Marker for Immutable {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            2 => Self::Boxed,
            3 => Self::Reference,
            4 => Self::Pointer,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

/////////////////////////

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum CowLike {
    Boxed = 2,
    #[default]
    Pointer = 4,
}
unsafe impl IsA<Variant> for CowLike {}
unsafe impl IsA<Immutable> for CowLike {}
unsafe impl IsA<Mutexed> for CowLike {}
impl Marker for CowLike {
    #[inline]
    fn from_u8(b: u8) -> Option<CowLike> {
        Some(match b {
            2 => Self::Boxed,
            4 => Self::Pointer,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Unclonable {
    Mutable = 1,
    #[default]
    Boxed = 2,
}
unsafe impl IsA<Variant> for Unclonable {}
unsafe impl IsA<Mutexed> for Unclonable {}
unsafe impl IsA<Dropped> for Unclonable {}
impl Marker for Unclonable {
    #[inline]
    fn from_u8(b: u8) -> Option<Unclonable> {
        Some(match b {
            1 => Self::Mutable,
            2 => Self::Boxed,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Accessor {
    Mutable = 1,
    #[default]
    Reference = 3,
}
unsafe impl IsA<Variant> for Accessor {}
unsafe impl IsA<Dropped> for Accessor {}
unsafe impl IsA<Borrowed> for Accessor {}
impl Marker for Accessor {
    #[inline]
    fn from_u8(b: u8) -> Option<Accessor> {
        Some(match b {
            1 => Self::Mutable,
            3 => Self::Reference,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum RefCount {
    Boxed = 2,
    #[default]
    Reference = 3,
}
unsafe impl IsA<Variant> for RefCount {}
unsafe impl IsA<Dropped> for RefCount {}
unsafe impl IsA<Immutable> for RefCount {}
impl Marker for RefCount {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            2 => Self::Boxed,
            3 => Self::Reference,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Clonable {
    Reference = 3,
    #[default]
    Pointer = 4,
}

unsafe impl IsA<Variant> for Clonable {}
unsafe impl IsA<Borrowed> for Clonable {}
unsafe impl IsA<Immutable> for Clonable {}
impl Marker for Clonable {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            3 => Self::Reference,
            4 => Self::Pointer,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Invalidated {
    Mutable = 1,
    #[default]
    Pointer = 4,
}
unsafe impl IsA<Variant> for Invalidated {}
unsafe impl IsA<Borrowed> for Invalidated {}
unsafe impl IsA<Mutexed> for Invalidated {}
impl Marker for Invalidated {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            1 => Self::Mutable,
            4 => Self::Pointer,
            _ => return None,
        })
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

//////////////////////////

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Mutable {
    #[default]
    Mutable = 1,
}
unsafe impl IsA<Unclonable> for Mutable {}
unsafe impl IsA<Accessor> for Mutable {}
unsafe impl IsA<Dropped> for Mutable {}
unsafe impl IsA<Mutexed> for Mutable {}
unsafe impl IsA<Borrowed> for Mutable {}
unsafe impl IsA<Invalidated> for Mutable {}
unsafe impl IsA<Variant> for Mutable {}
impl Marker for Mutable {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        match b {
            1 => Some(Self::Mutable),
            _ => None,
        }
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Boxed {
    #[default]
    Boxed = 2,
}
unsafe impl IsA<Unclonable> for Boxed {}
unsafe impl IsA<Dropped> for Boxed {}
unsafe impl IsA<Mutexed> for Boxed {}
unsafe impl IsA<RefCount> for Boxed {}
unsafe impl IsA<Immutable> for Boxed {}
unsafe impl IsA<CowLike> for Boxed {}
unsafe impl IsA<Variant> for Boxed {}
impl Marker for Boxed {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        match b {
            2 => Some(Self::Boxed),
            _ => None,
        }
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Reference {
    #[default]
    Reference = 3,
}
unsafe impl IsA<Variant> for Reference {}
unsafe impl IsA<Accessor> for Reference {}
unsafe impl IsA<Dropped> for Reference {}
unsafe impl IsA<Clonable> for Reference {}
unsafe impl IsA<Immutable> for Reference {}
unsafe impl IsA<RefCount> for Reference {}
unsafe impl IsA<Borrowed> for Reference {}
impl Marker for Reference {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        match b {
            3 => Some(Self::Reference),
            _ => None,
        }
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Pointer {
    #[default]
    Pointer = 4,
}
unsafe impl IsA<Variant> for Pointer {}
unsafe impl IsA<CowLike> for Pointer {}
unsafe impl IsA<Mutexed> for Pointer {}
unsafe impl IsA<Immutable> for Pointer {}
unsafe impl IsA<Borrowed> for Pointer {}
unsafe impl IsA<Invalidated> for Pointer {}
unsafe impl IsA<Clonable> for Pointer {}
impl Marker for Pointer {
    #[inline]
    fn from_u8(b: u8) -> Option<Self> {
        match b {
            4 => Some(Self::Pointer),
            _ => None,
        }
    }
    #[inline]
    fn to_u8(self) -> u8 {
        self as u8
    }
}

/////////////////

/// # Safety requirements
/// 1. The discriminants of Self must all be valid in E
pub unsafe trait IsA<E: Marker>: Marker {
    fn project(self) -> E {
        E::from_u8(self.to_u8()).unwrap()
    }
}

trait HasA<E: Marker>: Marker {}
impl<E: Marker, F: IsA<E>> HasA<E> for F {}

pub trait Marker: secret::Sealed + Default + fmt::Debug + PartialEq + Eq + Clone + Copy {
    const CHECK: secret::Secret = {
        if size_of::<Self>() != 1 || align_of::<Self>() != 1 {
            panic!();
        }
        secret::Secret
    };

    fn to_u8(self) -> u8;
    fn from_u8(b: u8) -> Option<Self>;

    fn as_a<T>(self) -> T
    where
        T: Marker,
        Self: IsA<T>,
    {
        self.project()
    }
}

unsafe impl<M: Marker> IsA<M> for M {}

use std::fmt;

mod secret {
    use super::*;
    pub trait Sealed {}
    impl Sealed for Variant {}

    impl Sealed for Borrowed {}
    impl Sealed for Mutexed {}
    impl Sealed for Immutable {}
    impl Sealed for Dropped {}

    impl Sealed for RefCount {}
    impl Sealed for Invalidated {}
    impl Sealed for Clonable {}
    impl Sealed for Unclonable {}
    impl Sealed for Accessor {}
    impl Sealed for CowLike {}

    impl Sealed for Mutable {}
    impl Sealed for Boxed {}
    impl Sealed for Reference {}
    impl Sealed for Pointer {}

    pub struct Secret;
}

////////////////////

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
