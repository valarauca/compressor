#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "RUSTC_NIGHTLY", feature(core_intrinsics))]

/// inconceivable is a macro which closely parallels `std::unreachable`, or `std::panic`.
///
/// The primary difference is that when this crate is
/// configured with the `ub_unreachable` option it will emit
/// the `core::hint::unreachable_unchecked` to hint
/// for the compiler to understand a condition should
/// never occur.
///
/// Generally compilers assume UB won't happen. This macro
/// offers the "best of both worlds", it provides a solid
/// way of asserting/testing behavior in debug builds, and
/// no cost in properly configured release builds.
#[macro_export]
macro_rules! inconceivable {
    () => {
        {
        #[cfg(all(not(feature="std"), feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27"))]
        {
            unsafe{ ::core::hint::unreachable_unchecked() }
        }

        #[cfg(all(feature = "std", feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27"))]
        {
            unsafe{ ::std::hint::unreachable_unchecked() }
        }


        #[cfg(not(all(feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27")))]
        {
            unreachable!()
        }
        }
    };
    ($msg: expr) => {
        {
        #[cfg(all(not(feature="std"), feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27"))]
        {
            unsafe{ ::core::hint::unreachable_unchecked() }
        }

        #[cfg(all(feature = "std", feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27"))]
        {
            unsafe{ ::std::hint::unreachable_unchecked() }
        }


        #[cfg(not(all(feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27")))]
        {
            unreachable!($msg)
        }
        }
    };
    ($msg: expr,) => {
        {
        #[cfg(all(not(feature="std"), feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27"))]
        {
            unsafe{ ::core::hint::unreachable_unchecked() }
        }

        #[cfg(all(feature = "std", feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27"))]
        {
            unsafe{ ::std::hint::unreachable_unchecked() }
        }


        #[cfg(not(all(feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27")))]
        {
            unreachable!($msg)
        }
        }
    };
    ($fmt: expr, $($arg:tt)*) => {
        {
        #[cfg(all(not(feature="std"), feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27"))]
        {
            unsafe{ ::core::hint::unreachable_unchecked() }
        }

        #[cfg(all(feature = "std", feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27"))]
        {
            unsafe{ ::std::hint::unreachable_unchecked() }
        }

        #[cfg(not(all(feature = "ub_unreachable", feature = "RUSTC_VERSION_GE_1_27")))]
        {
            unreachable!($fmt, $($arg)*)
        }
        }
    };
}

/// numbers concerns dereferencing values on different platforms
pub mod numbers;

/// intrinsics handle a lot of the semantics of branch hinting
pub mod intrinsics;

/// intrinsics for cache prefetch hinting
pub mod prefetch;

/// mem handles the normal `std::`/`core::` imports.
pub mod mem;
