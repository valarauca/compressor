/// walks a buffer and hints to processor that we may want the whole thing
/// in cache.
///
/// Linearly scanning a buffer is well predictable, so this doesn't do
/// much, but it also doesn't cost a lot.
#[allow(dead_code, unused_variables, unreachable_code)]
#[inline(never)]
pub fn prefetch_buffer<T: Sized>(buffer: &[T]) {
    #[cfg(not(feature = "prefetch_hints"))]
    {
        // if prefetching is not enabled, don't do anything
        return;
    }

    for chunk in buffer.chunks(item_per_cache_line::<T>()) {
        prefetch(chunk.as_ptr());
    }
}

/// prefetch will provide some hinting to the CPU we want data within the cache
/// it does not specify which cache. This may only work on
#[allow(dead_code, unused_variables)]
#[inline(always)]
pub fn prefetch<T>(ptr: *const T) {
    #[cfg(all(
        target_arch = "x86_64",
        feature = "prefetch_hints",
        not(feature = "RUSTC_NIGHTLY")
    ))]
    {
        use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T2};

        // no need to check if sse is active/un-active. SSE (and SSE2) are part of x86_64's ABI
        unsafe { _mm_prefetch(ptr as *const i8, _MM_HINT_T2) };
    }

    #[cfg(all(feature = "RUSTC_NIGHTLY", feature = "prefetch_hints"))]
    {
        ::core::intrinsics::prefetch_read_data(ptr, 0);
    }
}

/// Assuming you're working with an array, this calculates the (rounded down) number
/// of items within a cache line. YMMV with alignment.
#[inline(always)]
fn item_per_cache_line<T: Sized>() -> usize {
    use core::mem::size_of;
    // cache lines are 64bytes.
    // The author is aware they are not sometimes for example:
    // - https://www.mono-project.com/news/2016/09/12/arm64-icache/
    // - https://reviews.llvm.org/rG457ddd311a164b31c7ef431abd4fd5dba84683f4
    //
    // but honestly, it doesn't matter. prefetching hints can be
    // discarded anyways
    let cache_line_size = 64usize;
    let item_size = size_of::<T>();
    if item_size == 0 || item_size >= cache_line_size {
        return 1;
    }
    let size = (0..cache_line_size)
        .map(|i| (i, i * item_size))
        .filter(|(_, s)| *s >= cache_line_size)
        .map(|(i, _)| i)
        .next();
    match size {
        Option::None => inconceivable!(),
        Option::Some(x) => x,
    }
}
