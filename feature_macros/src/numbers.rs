#[cfg(not(feature = "std"))]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::hash::Hash;

#[cfg(not(feature = "std"))]
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};
#[cfg(not(feature = "std"))]
use core::ptr::{read, read_unaligned};
#[cfg(feature = "std")]
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};
#[cfg(feature = "std")]
use std::ptr::{read, read_unaligned};

use super::mem::{align_of, size_of, transmute};

/// Num wraps a primative value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Num<T>
where
    T: Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Default
        + Shr<i32, Output = T>
        + Shl<i32, Output = T>
        + BitXor<T, Output = T>
        + BitAnd<T, Output = T>
        + BitOr<T, Output = T>
        + Not
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShrAssign<i32>
        + ShlAssign<i32>,
{
    data: T,
}

impl<T> From<T> for Num<T>
where
    T: Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Default
        + Shr<i32, Output = T>
        + Shl<i32, Output = T>
        + BitXor<T, Output = T>
        + BitAnd<T, Output = T>
        + BitOr<T, Output = T>
        + Not
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShrAssign<i32>
        + ShlAssign<i32>,
{
    #[inline(always)]
    fn from(data: T) -> Num<T> {
        Num { data: data }
    }
}
macro_rules! implement_num_core_op_trait {
    ($trait_norm: ident => $func_norm: ident ; $trait_asg: ident => $func_asg: ident) => {
        impl<T> $trait_norm for Num<T>
        where
            T: Copy
                + Clone
                + PartialEq
                + Eq
                + PartialOrd
                + Ord
                + Hash
                + Default
                + Shr<i32, Output = T>
                + Shl<i32, Output = T>
                + BitXor<T, Output = T>
                + BitAnd<T, Output = T>
                + BitOr<T, Output = T>
                + Not
                + BitAndAssign
                + BitOrAssign
                + BitXorAssign
                + ShrAssign<i32>
                + ShlAssign<i32>,
        {
            type Output = Self;

            #[inline(always)]
            fn $func_norm(self, other: Self) -> Self {
                Num::from(self.data.$func_norm(other.data))
            }
        }
        impl<T> $trait_norm<T> for Num<T>
        where
            T: Copy
                + Clone
                + PartialEq
                + Eq
                + PartialOrd
                + Ord
                + Hash
                + Default
                + Shr<i32, Output = T>
                + Shl<i32, Output = T>
                + BitXor<T, Output = T>
                + BitAnd<T, Output = T>
                + BitOr<T, Output = T>
                + Not
                + BitAndAssign
                + BitOrAssign
                + BitXorAssign
                + ShrAssign<i32>
                + ShlAssign<i32>,
        {
            type Output = Self;

            #[inline(always)]
            fn $func_norm(self, other: T) -> Self {
                let x: T = self.data.$func_norm(other);
                Num::from(x)
            }
        }
        impl<T> $trait_asg for Num<T>
        where
            T: Copy
                + Clone
                + PartialEq
                + Eq
                + PartialOrd
                + Ord
                + Hash
                + Default
                + Shr<i32, Output = T>
                + Shl<i32, Output = T>
                + BitXor<T, Output = T>
                + BitAnd<T, Output = T>
                + BitOr<T, Output = T>
                + Not
                + BitAndAssign
                + BitOrAssign
                + BitXorAssign
                + ShrAssign<i32>
                + ShlAssign<i32>,
        {
            #[inline(always)]
            fn $func_asg(&mut self, other: Self) {
                self.data.$func_asg(other.data);
            }
        }
        impl<T> $trait_asg<T> for Num<T>
        where
            T: Copy
                + Clone
                + PartialEq
                + Eq
                + PartialOrd
                + Ord
                + Hash
                + Default
                + Shr<i32, Output = T>
                + Shl<i32, Output = T>
                + BitXor<T, Output = T>
                + BitAnd<T, Output = T>
                + BitOr<T, Output = T>
                + Not
                + BitAndAssign
                + BitOrAssign
                + BitXorAssign
                + ShrAssign<i32>
                + ShlAssign<i32>,
        {
            #[inline(always)]
            fn $func_asg(&mut self, other: T) {
                self.data.$func_asg(other);
            }
        }
    };
}

impl<T> Shr<i32> for Num<T>
where
    T: Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Default
        + Shr<i32, Output = T>
        + Shl<i32, Output = T>
        + BitXor<T, Output = T>
        + BitAnd<T, Output = T>
        + BitOr<T, Output = T>
        + Not
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShrAssign<i32>
        + ShlAssign<i32>,
{
    type Output = Self;

    #[inline(always)]
    fn shr(self, other: i32) -> Self {
        Num::from(self.data.shr(other))
    }
}
impl<T> ShrAssign<i32> for Num<T>
where
    T: Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Default
        + Shr<i32, Output = T>
        + Shl<i32, Output = T>
        + BitXor<T, Output = T>
        + BitAnd<T, Output = T>
        + BitOr<T, Output = T>
        + Not
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShrAssign<i32>
        + ShlAssign<i32>,
{
    #[inline(always)]
    fn shr_assign(&mut self, other: i32) {
        self.data.shr_assign(other)
    }
}

impl<T> Shl<i32> for Num<T>
where
    T: Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Default
        + Shl<i32, Output = T>
        + Shr<i32, Output = T>
        + BitXor<T, Output = T>
        + BitAnd<T, Output = T>
        + BitOr<T, Output = T>
        + Not
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShlAssign<i32>
        + ShrAssign<i32>,
{
    type Output = Self;

    #[inline(always)]
    fn shl(self, other: i32) -> Self {
        Num::from(self.data.shl(other))
    }
}
impl<T> ShlAssign<i32> for Num<T>
where
    T: Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Default
        + Shl<i32, Output = T>
        + Shr<i32, Output = T>
        + BitXor<T, Output = T>
        + BitAnd<T, Output = T>
        + BitOr<T, Output = T>
        + Not
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShlAssign<i32>
        + ShrAssign<i32>,
{
    #[inline(always)]
    fn shl_assign(&mut self, other: i32) {
        self.data.shl_assign(other)
    }
}

implement_num_core_op_trait!(BitXor => bitxor; BitXorAssign => bitxor_assign);
implement_num_core_op_trait!(BitAnd => bitand; BitAndAssign => bitand_assign);
implement_num_core_op_trait!(BitOr => bitor; BitOrAssign => bitor_assign);

/// PrimativeNumber is a general trait that has a load of requirements
/// to effectively ensure it can only be implemented on primative
/// integer values.
pub trait PrimativeNumber:
    Clone
    + Copy
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
    + Default
    + From<<Self as PrimativeNumber>::Primative>
{
    /// The interger type we're encapsulating
    type Primative: Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Default
        + Shr<i32, Output = Self::Primative>
        + Shl<i32, Output = Self::Primative>
        + BitXor<Self::Primative, Output = Self::Primative>
        + BitAnd<Self::Primative, Output = Self::Primative>
        + BitOr<Self::Primative, Output = Self::Primative>
        + Not
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShrAssign<i32>
        + ShlAssign<i32>;

    fn max() -> Self::Primative;
    fn min() -> Self::Primative;

    /// destroys value, returns interior
    fn inner(self) -> Self::Primative;

    /// swaps the bytes of a value
    fn swap_bytes(self) -> Self;

    fn rotate_left(self, arg: u32) -> Self;
    fn rotate_right(self, arg: u32) -> Self;

    fn wrapping_add<A>(self, arg: A) -> Self
    where
        Self: From<A>;

    fn wrapping_sub<A>(self, arg: A) -> Self
    where
        Self: From<A>;

    fn wrapping_mul<A>(self, arg: A) -> Self
    where
        Self: From<A>;

    /// This function will perform a byte swap if and only if the platform is little endian
    #[allow(dead_code)]
    #[inline]
    fn swap_if_platform_is_little_endian(self) -> Self {
        #[cfg(target_endian = "little")]
        {
            self.swap_bytes()
        }

        #[cfg(target_endian = "big")]
        {
            self
        }
    }

    /// This function will perform a byte swap if and only if the platform is big endian
    #[allow(dead_code)]
    #[inline]
    fn swap_if_platform_is_big_endian(self) -> Self {
        #[cfg(target_endian = "big")]
        {
            self.swap_bytes()
        }

        #[cfg(target_endian = "little")]
        {
            self
        }
    }

    /// This function handles a mess of platform specific details concerning
    /// aligned and non-aligned loads. This will return a value in the native
    /// endianess
    #[allow(dead_code)]
    #[inline]
    fn read_value(arg: &[u8]) -> Self {
        #[cfg(any(
            target_arch = "arm",
            target_arch = "aarch64",
            target_arch = "powerpc",
            target_arch = "mips"
        ))]
        {
            // TODO: benchmark the internal conditional alignment check

            Self::from(alignment_matters_load::<Self::Primative>(arg))
        }

        #[cfg(any(target_arch = "x86", target_arch = "powerpc64", target_arch = "x86_64"))]
        {
            if size_of::<Self::Primative>() >= 16 {
                // no platform handles SIMD size loads well
                Self::from(alignment_matters_load::<Self::Primative>(arg))
            } else {
                // as far as I can tell powerpc64 will handle unaligned loads fine.
                // but this maybe a mistake.
                Self::from(screw_alignment_load::<Self::Primative>(arg))
            }
        }
    }

    /// reads a value into big endian.
    ///
    /// when targetting a platform with native little endian
    /// format it will emit a byte swap
    #[allow(dead_code)]
    #[inline]
    fn read_value_be(arg: &[u8]) -> Self {
        Self::read_value(arg).swap_if_platform_is_little_endian()
    }

    /// reads a value into little endian.
    ///
    /// when targetting a platform with native big endian
    /// format it will emit a byte swap
    #[allow(dead_code)]
    #[inline]
    fn read_value_le(arg: &[u8]) -> Self {
        Self::read_value(arg).swap_if_platform_is_big_endian()
    }
}

/// a function for doing alignment checking
#[allow(dead_code)]
#[inline(always)]
fn is_aligned<T: Copy>(arg: *const u8) -> bool {
    let address: usize = unsafe { transmute(arg) };
    (address & (align_of::<T>() - 1usize)) == 0
}

/// general function for handling all the semantics of loading on platforms
/// where alignment matters
#[allow(dead_code)]
#[inline(always)]
fn alignment_matters_load<T: Copy>(arg: &[u8]) -> T {
    #[cfg(not(feature = "unbound"))]
    {
        if arg.len() < size_of::<T>() {
            panic!("out of bounds memory access");
        }
    }

    unsafe {
        if is_aligned::<T>(arg.as_ptr()) {
            read::<T>(transmute::<*const u8, *const T>(arg.as_ptr()))
        } else {
            read_unaligned::<T>(transmute::<*const u8, *const T>(arg.as_ptr()))
        }
    }
}

/// general function for handling all the semantics of loading on platforms
/// where alignment doesn't matter
#[allow(dead_code)]
#[inline(always)]
fn screw_alignment_load<T: Copy>(arg: &[u8]) -> T {
    #[cfg(not(feature = "unbound"))]
    {
        if arg.len() < size_of::<T>() {
            panic!("out of bounds memory access");
        }
    }

    unsafe { read::<T>(transmute::<*const u8, *const T>(arg.as_ptr())) }
}

macro_rules! implement_primative_number {
    ($kind: ident) => {
        impl PrimativeNumber for Num<$kind> {
            type Primative = $kind;

            #[inline(always)]
            fn inner(self) -> $kind {
                self.data
            }

            #[inline(always)]
            fn max() -> $kind {
                #[cfg(not(feature = "std"))]
                use core::$kind::MAX;
                #[cfg(feature = "std")]
                use std::$kind::MAX;

                MAX
            }

            #[inline(always)]
            fn min() -> $kind {
                #[cfg(not(feature = "std"))]
                use core::$kind::MIN;
                #[cfg(feature = "std")]
                use std::$kind::MIN;

                MIN
            }

            #[inline(always)]
            fn swap_bytes(self) -> Num<$kind> {
                Num::from(self.data.swap_bytes())
            }

            #[inline(always)]
            fn rotate_right(self, arg: u32) -> Num<$kind> {
                Num::from(self.data.rotate_right(arg))
            }

            #[inline(always)]
            fn rotate_left(self, arg: u32) -> Num<$kind> {
                Num::from(self.data.rotate_left(arg))
            }

            #[inline(always)]
            fn wrapping_add<G>(self, arg: G) -> Num<$kind>
            where
                Num<$kind>: From<G>,
            {
                let x: Num<$kind> = <Num<$kind> as From<G>>::from(arg);
                let y: $kind = self.data.wrapping_add(x.data);
                <Num<$kind> as From<$kind>>::from(y)
            }

            #[inline(always)]
            fn wrapping_sub<G>(self, arg: G) -> Num<$kind>
            where
                Self: From<G>,
            {
                let x: Num<$kind> = <Num<$kind> as From<G>>::from(arg);
                let y: $kind = self.data.wrapping_sub(x.data);
                <Num<$kind> as From<$kind>>::from(y)
            }

            #[inline(always)]
            fn wrapping_mul<G>(self, arg: G) -> Num<$kind>
            where
                Self: From<G>,
            {
                let x: Num<$kind> = <Num<$kind> as From<G>>::from(arg);
                let y: $kind = self.data.wrapping_mul(x.data);
                <Num<$kind> as From<$kind>>::from(y)
            }
        }
    };
}

implement_primative_number!(u8);
implement_primative_number!(i8);
implement_primative_number!(u16);
implement_primative_number!(i16);
implement_primative_number!(u32);
implement_primative_number!(i32);
implement_primative_number!(u64);
implement_primative_number!(i64);
implement_primative_number!(usize);
implement_primative_number!(isize);
#[cfg(feature = "RUSTC_VERSION_GE_1_26")]
implement_primative_number!(u128);
#[cfg(feature = "RUSTC_VERSION_GE_1_26")]
implement_primative_number!(i128);
