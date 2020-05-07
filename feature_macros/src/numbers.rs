use super::unstd::hash::Hash;
use super::unstd::mem::{align_of, size_of, transmute};
use super::unstd::ops::*;
use super::unstd::ptr::{read, read_unaligned};

/// These are the traits every basic number implemetns
pub trait BasicNumber:
    Copy
    + Clone
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
    + Default
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + BitAnd<Self, Output = Self>
    + BitAndAssign<Self>
    + BitOr<Self, Output = Self>
    + BitOrAssign<Self>
    + BitXor<Self, Output = Self>
    + BitXorAssign<Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + Mul<Self, Output = Self>
    + MulAssign<Self>
    + Not<Output = Self>
    + Rem<Self, Output = Self>
    + RemAssign<Self>
    + Shl<i8, Output = Self>
    + Shl<u8, Output = Self>
    + Shl<i16, Output = Self>
    + Shl<u16, Output = Self>
    + Shl<i32, Output = Self>
    + Shl<u32, Output = Self>
    + Shl<i64, Output = Self>
    + Shl<u64, Output = Self>
    + Shl<i64, Output = Self>
    + Shl<isize, Output = Self>
    + Shl<usize, Output = Self>
    + ShlAssign<i8>
    + ShlAssign<u8>
    + ShlAssign<i16>
    + ShlAssign<u16>
    + ShlAssign<i32>
    + ShlAssign<u32>
    + ShlAssign<i64>
    + ShlAssign<u64>
    + ShlAssign<i64>
    + ShlAssign<isize>
    + ShlAssign<usize>
    + Shr<i8, Output = Self>
    + Shr<u8, Output = Self>
    + Shr<i16, Output = Self>
    + Shr<u16, Output = Self>
    + Shr<i32, Output = Self>
    + Shr<u32, Output = Self>
    + Shr<i64, Output = Self>
    + Shr<u64, Output = Self>
    + Shr<i64, Output = Self>
    + Shr<isize, Output = Self>
    + Shr<usize, Output = Self>
    + ShrAssign<i8>
    + ShrAssign<u8>
    + ShrAssign<i16>
    + ShrAssign<u16>
    + ShrAssign<i32>
    + ShrAssign<u32>
    + ShrAssign<i64>
    + ShrAssign<u64>
    + ShrAssign<i64>
    + ShrAssign<isize>
    + ShrAssign<usize>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
{
}
impl BasicNumber for i8 {}
impl BasicNumber for u8 {}
impl BasicNumber for i16 {}
impl BasicNumber for u16 {}
impl BasicNumber for i32 {}
impl BasicNumber for u32 {}
impl BasicNumber for i64 {}
impl BasicNumber for u64 {}
impl BasicNumber for usize {}
impl BasicNumber for isize {}

/// Num wraps a primative value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Num<T: BasicNumber> {
    data: T,
}
impl<T: BasicNumber> Default for Num<T> {
    #[inline(always)]
    fn default() -> Num<T> {
        Num::from(T::default())
    }
}
impl<T: BasicNumber> Not for Num<T> {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        Num::from(<T as Not>::not(self.data))
    }
}
impl<T: BasicNumber> From<T> for Num<T> {
    #[inline(always)]
    fn from(data: T) -> Num<T> {
        Num { data: data }
    }
}
impl<T: BasicNumber> From<&T> for Num<T> {
    #[inline(always)]
    fn from(data: &T) -> Num<T> {
        Num { data: *data }
    }
}
impl<T: BasicNumber> From<&Num<T>> for Num<T>
where
    Self: PrimativeNumber<Primative = T>,
{
    #[inline(always)]
    fn from(data: &Num<T>) -> Num<T> {
        Num {
            data: <Self as PrimativeNumber>::inner(*data),
        }
    }
}
impl<T: BasicNumber> From<&&T> for Num<T> {
    #[inline(always)]
    fn from(data: &&T) -> Num<T> {
        Num { data: **data }
    }
}
impl<T: BasicNumber> From<&&Num<T>> for Num<T>
where
    Self: PrimativeNumber<Primative = T>,
{
    #[inline(always)]
    fn from(data: &&Num<T>) -> Num<T> {
        Num {
            data: <Num<T> as PrimativeNumber>::inner(**data),
        }
    }
}

macro_rules! implement_simple {
    ($(($trait_name: ident => $func_name: ident)),* $(,)*) => {
        $(
        impl<T: BasicNumber> $trait_name<Self> for Num<T> {
            type Output = Self;
            #[inline(always)]
            fn $func_name(self, other: Self) -> Self {
                Num::from(<T as $trait_name>::$func_name(self.data, other.data))
            }
        }
        )*
    }
}

implement_simple! {
    (Add => add),
    (BitAnd => bitand),
    (BitOr => bitor),
    (BitXor => bitxor),
    (Div => div),
    (Mul => mul),
    (Rem => rem),
    (Sub => sub),
}

macro_rules! implement_assign {
    ($(($trait_name: ident => $func_name: ident)),* $(,)*) => {
        $(
        impl<T: BasicNumber> $trait_name<Self> for Num<T> {
            #[inline(always)]
            fn $func_name(&mut self, other: Self) {
                <T as $trait_name>::$func_name(&mut self.data, other.data);
            }
        }
        )*
    }
}

implement_assign! {
    (AddAssign => add_assign),
    (BitAndAssign => bitand_assign),
    (BitOrAssign => bitor_assign),
    (BitXorAssign => bitxor_assign),
    (DivAssign => div_assign),
    (MulAssign => mul_assign),
    (RemAssign => rem_assign),
    (SubAssign => sub_assign),
}

macro_rules! implement_shifts {
    ( $(($trait_name: ident => $func_name: ident => { $($kind: ty),* })),* $(,)*) => {
        $(
            $(
                impl<T: BasicNumber> $trait_name<$kind> for Num<T> {
                    type Output = Self;
                    #[inline(always)]
                    fn $func_name(self, rhs: $kind) -> Self {
                        Num::from(<T as $trait_name<$kind>>::$func_name(self.data, rhs))
                    }
                }
            )*
        )*
    }
}
implement_shifts! {
    (Shr => shr => { u8, i8, u16, i16, u32, i32, u64, i64, usize, isize }),
    (Shl => shl => { u8, i8, u16, i16, u32, i32, u64, i64, usize, isize }),
}

macro_rules! implement_shift_assign {
    ( $(($trait_name: ident => $func_name: ident => { $($kind: ty),* })),* $(,)*) => {
        $(
            $(
                impl<T: BasicNumber> $trait_name<$kind> for Num<T> {
                    #[inline(always)]
                    fn $func_name(&mut self, rhs: $kind) {
                        <T as $trait_name<$kind>>::$func_name(&mut self.data, rhs)
                    }
                }
            )*
        )*
    }
}
implement_shift_assign! {
    (ShrAssign => shr_assign => { u8, i8, u16, i16, u32, i32, u64, i64, usize, isize }),
    (ShlAssign => shl_assign => { u8, i8, u16, i16, u32, i32, u64, i64, usize, isize }),
}

macro_rules! implement_num_from_boilerplate {
    ( $(($base_kind: ty => { $($from_kind: ty),* })),* $(,)* ) => {
        $(
            $(
                impl From<$from_kind> for Num<$base_kind> {
                    #[inline(always)]
                    fn from(arg: $from_kind) -> Num<$base_kind> {
                        Num::from(arg as $base_kind)
                    }
                }
            )*
        )*
    }
}

implement_num_from_boilerplate! {
    (u8 => { i8, u16, i16, u32, i32, u64, i64, usize, isize }),
    (i8 => { u8, u16, i16, u32, i32, u64, i64, usize, isize }),
    (i16 => { u8, i8, u16, u32, i32, u64, i64, usize, isize }),
    (u16 => { u8, i8, i16, u32, i32, u64, i64, usize, isize }),
    (i32 => { u8, i8, u16, i16, u32, u64, i64, usize, isize }),
    (u32 => { u8, i8, u16, i16, i32, u64, i64, usize, isize }),
    (i64 => { u8, i8, u16, i16, u32, i32, u64, usize, isize }),
    (u64 => { u8, i8, u16, i16, u32, i32, i64, usize, isize }),
    (isize => { u8, i8, u16, i16, u32, i32, u64, i64, usize }),
    (usize => { u8, i8, u16, i16, u32, i32, u64, i64, isize }),
}

/// PrimativeNumber is a general trait that has a load of requirements
/// to effectively ensure it can only be implemented on primative
/// integer values.
pub trait PrimativeNumber: BasicNumber + From<<Self as PrimativeNumber>::Primative> {
    /// The interger type we're encapsulating
    type Primative: BasicNumber;

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
        impl BasicNumber for Num<$kind> {}
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
