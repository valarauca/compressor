#[cfg(not(feature = "std"))]
use core::intrinsics::copy_nonoverlapping;
#[cfg(all(
    not(feature = "std"),
    feature = "RUSTC_NIGHTLY",
    feature = "branch_hints"
))]
use core::intrinsics::{likely, unlikely};

#[cfg(feature = "std")]
use std::intrinsics::copy_nonoverlapping;
#[cfg(all(feature = "std", feature = "RUSTC_NIGHTLY", feature = "branch_hints"))]
use std::intrinsics::{likely, unlikely};

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn memcp<T: Sized>(src: *const T, dst: *mut T, len: usize) {
    copy_nonoverlapping(src, dst, len);
}

/// this will emit a branch hint if you're on nightly & use the `branch_hints` feature
#[allow(dead_code)]
#[inline(always)]
pub fn hint_likely(b: bool) -> bool {
    #[cfg(all(feature = "RUSTC_NIGHTLY", feature = "branch_hints"))]
    {
        likely(b)
    }

    #[cfg(any(not(feature = "RUSTC_NIGHTLY"), not(feature = "branch_hints")))]
    {
        b
    }
}

/// this will emit a branch hint if you're on nightly & use the `branch_hints` feature
#[allow(dead_code)]
#[inline(always)]
pub fn hint_unlikely(b: bool) -> bool {
    #[cfg(all(feature = "RUSTC_NIGHTLY", feature = "branch_hints"))]
    {
        unlikely(b)
    }

    #[cfg(any(not(feature = "RUSTC_NIGHTLY"), not(feature = "branch_hints")))]
    {
        b
    }
}
