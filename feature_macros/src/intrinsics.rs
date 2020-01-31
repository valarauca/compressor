/// this will emit a branch hint if you're on nightly & use the `branch_hints` feature
#[allow(dead_code)]
#[inline(always)]
pub fn hint_likely(b: bool) -> bool {
    #[cfg(all(feature = "RUSTC_NIGHTLY", feature = "branch_hints"))]
    {
        ::core::intrinsics::likely(b)
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
        ::core::intrinsics::unlikely(b)
    }

    #[cfg(any(not(feature = "RUSTC_NIGHTLY"), not(feature = "branch_hints")))]
    {
        b
    }
}
