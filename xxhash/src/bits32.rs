use super::feature_macros::intrinsics::{hint_likely, memcp};
use super::feature_macros::numbers::{Num, PrimativeNumber};
use super::feature_macros::prefetch::prefetch_buffer;

#[cfg(not(feature = "std"))]
use core::cmp::min;
#[cfg(not(feature = "std"))]
use core::hash::Hasher;
#[cfg(feature = "std")]
use std::cmp::min;
#[cfg(feature = "std")]
use std::hash::Hasher;

#[cfg(not(feature = "std"))]
use core::slice::from_raw_parts;
#[cfg(feature = "std")]
use std::slice::from_raw_parts;

const PRIME32_1: u32 = 2654435761u32;
const PRIME32_2: u32 = 2246822519u32;
const PRIME32_3: u32 = 3266489917u32;
const PRIME32_4: u32 = 668265263u32;
const PRIME32_5: u32 = 374761393u32;

/// the basic convolution each value will undergo
#[inline(always)]
fn xxh32_round(seed: Num<u32>, input: Num<u32>) -> Num<u32> {
    seed.wrapping_add(input.wrapping_mul(PRIME32_2))
        .rotate_left(13)
        .wrapping_mul(PRIME32_1)
}

#[inline(always)]
fn xxh32_avalanche(seed: Num<u32>) -> Num<u32> {
    let mut seed = seed;
    seed ^= seed >> 15;
    seed = seed.wrapping_mul(PRIME32_2);
    seed ^= seed >> 13;
    seed = seed.wrapping_mul(PRIME32_3);
    seed ^= seed >> 16;
    seed
}

/// finalize works on the last <15 bytes of the buffer
#[inline(always)]
fn xxh32_finalize(seed: Num<u32>, buffer: &[u8]) -> Num<u32> {
    // this works on the last <3 bytes of the buffer
    #[inline(always)]
    fn inner_most_fold(hash: Num<u32>, arg: &u8) -> Num<u32> {
        let ingest = (*arg as u32).wrapping_mul(PRIME32_5);
        hash.wrapping_add(ingest)
            .rotate_left(11)
            .wrapping_mul(PRIME32_1)
    }

    // this is invoked on the last few 4 byte wide chunks
    #[inline(always)]
    fn fold_chunk(hash: Num<u32>, arg: &[u8]) -> Num<u32> {
        let length = arg.len();
        debug_assert!(
            length <= 4 && length > 0,
            "length can only ever be 4, 3, 2, or 1"
        );
        if length == 4 {
            let ingest = Num::<u32>::read_value_le(arg).wrapping_mul(PRIME32_3);
            hash.wrapping_add(ingest)
                .rotate_left(17)
                .wrapping_mul(PRIME32_4)
        } else {
            arg.iter().fold(hash, inner_most_fold)
        }
    }

    xxh32_avalanche(buffer.chunks(4).fold(seed, fold_chunk))
}

/// this is the reference xxhash 32bit implemenation
#[allow(dead_code)]
fn xxh32_reference(seed: Num<u32>, buffer: &[u8]) -> Num<u32> {
    /*
     * please note:
     *    This code is not super convulted b/c the chunks
     *    API provides assertions about the size of slices
     *    ahead of time. So (nearly) all bounds checking is
     *    eliminated.
     *
     *    When `unbounded` is used the bounds check within
     *    `Num::<u32>::read_value_le` is removed.
     *
     */

    /// inteiror round is the initial bit mixing that values under-go
    /// when they're derefereneced into to the hash's processing
    /// mechanicism.
    #[inline(always)]
    fn interior_round(seed: Num<u32>, buffer: &[u8]) -> Num<u32> {
        debug_assert!(buffer.len() == 4, "buffer should always be 4 elements");
        xxh32_round(seed, Num::<u32>::read_value_le(buffer))
    }

    // this is an uglier lambda that is used to consume 16 bytes
    // at a time.
    #[inline(always)]
    fn big_chunk(v: [Num<u32>; 4], chunk: &[u8]) -> [Num<u32>; 4] {
        debug_assert!(chunk.len() == 16);
        [
            interior_round(v[0], &chunk[0..4]),
            interior_round(v[1], &chunk[4..8]),
            interior_round(v[2], &chunk[8..12]),
            interior_round(v[3], &chunk[12..16]),
        ]
    }

    let hash = if buffer.len() >= 16 {
        // this function may do nothing depending on the feature flag
        prefetch_buffer(buffer);
        let fold_start: [Num<u32>; 4] = [
            seed.wrapping_add(PRIME32_1).wrapping_add(PRIME32_2),
            seed.wrapping_add(PRIME32_2),
            seed,
            seed.wrapping_sub(PRIME32_1),
        ];
        let output = buffer
            .chunks(16)
            .filter(|chunk| hint_likely(chunk.len() == 16))
            .fold(fold_start, big_chunk);
        output[0]
            .rotate_left(1)
            .wrapping_add(output[1].rotate_left(7))
            .wrapping_add(output[2].rotate_left(12))
            .wrapping_add(output[3].rotate_left(18))
            .wrapping_add(buffer.len() as u32)
    } else {
        seed.wrapping_add(PRIME32_5)
            .wrapping_add(buffer.len() as u32)
    };
    // weird modulus operation to get the last ~15 bytes of the buffer
    let last = buffer.len() & (<Num<usize> as PrimativeNumber>::max() - 15usize);
    xxh32_finalize(hash, &buffer[last..])
}

/// reference implementation of xxhash32
#[inline(never)]
pub fn xxhash32_reference(seed: u32, buffer: &[u8]) -> u32 {
    xxh32_reference(Num::from(seed), buffer).inner()
}

/// xxhash32_ffi is exposed for consumption by the FFI into C/C++ projects
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn xxhash32_ffi(seed: u32, ptr: *const u8, len: usize) -> u32 {
    xxh32_reference(Num::from(seed), from_raw_parts(ptr, len)).inner()
}

/// XXHash32 is an implementation of two-x hash.
/// this structure offers a streaming implementation
#[derive(Clone)]
pub struct XXHash32 {
    interior_length: usize,
    total_length: usize,
    seed: Num<u32>,
    state: [Num<u32>; 4],
    interior_buffer: [u8; 16],
}
impl Default for XXHash32 {
    fn default() -> XXHash32 {
        XXHash32::new()
    }
}
impl XXHash32 {
    /// Builds an XXHash32 with a seed of `0`, this is identical to `Default`
    pub fn new() -> XXHash32 {
        XXHash32::with_seed(0)
    }

    /// construct with a deterministic seed
    pub fn with_seed(seed: u32) -> XXHash32 {
        let seed = Num::<u32>::from(seed);
        XXHash32 {
            interior_length: 0,
            total_length: 0,
            seed: Num::<u32>::from(seed),
            state: [
                seed.wrapping_add(PRIME32_1).wrapping_add(PRIME32_2),
                seed.wrapping_add(PRIME32_2),
                seed,
                seed.wrapping_sub(PRIME32_1),
            ],
            interior_buffer: [0u8; 16],
        }
    }

    fn perform_finish(&self) -> u32 {
        debug_assert!(self.interior_length <= 16);

        let hash = if self.total_length >= 16 {
            self.state[0]
                .rotate_left(1)
                .wrapping_add(self.state[1].rotate_left(7))
                .wrapping_add(self.state[2].rotate_left(12))
                .wrapping_add(self.state[3].rotate_left(18))
                .wrapping_add(self.total_length as u32)
        } else {
            self.seed
                .wrapping_add(PRIME32_5)
                .wrapping_add(self.total_length as u32)
        };
        xxh32_finalize(
            Num::<u32>::from(hash),
            &self.interior_buffer[0..self.interior_length],
        )
        .inner()
    }

    // attempts to consume the input to flush our internal buffer
    fn maybe_consume<'a>(&mut self, slice: &'a [u8]) -> &'a [u8] {
        if self.is_empty() || slice.is_empty() {
            slice
        } else {
            let minimum = min(slice.len(), self.avaliable());
            let (process, remaining) = slice.split_at(minimum);
            self.copy_into_internal(process);
            self.maybe_consume(remaining)
        }
    }

    fn consume<'a>(&mut self, arg: &'a [u8]) {
        let fold_lambda = |([v1, v2, v3, v4], _, len): ([Num<u32>; 4], Option<&'a [u8]>, usize),
                           chunk: &'a [u8]|
         -> ([Num<u32>; 4], Option<&'a [u8]>, usize) {
            if chunk.len() == 16 {
                (
                    [
                        xxh32_round(v1, Num::<u32>::read_value_le(&chunk[0..4])),
                        xxh32_round(v2, Num::<u32>::read_value_le(&chunk[4..8])),
                        xxh32_round(v3, Num::<u32>::read_value_le(&chunk[8..12])),
                        xxh32_round(v4, Num::<u32>::read_value_le(&chunk[12..16])),
                    ],
                    None,
                    len + 16,
                )
            } else {
                ([v1, v2, v3, v4], Some(chunk), len)
            }
        };
        let folder_state = (
            [self.state[0], self.state[1], self.state[2], self.state[3]],
            None,
            0,
        );
        let ([v1, v2, v3, v4], leftover, len) = self
            .maybe_consume(arg)
            .chunks(16)
            .fold(folder_state, fold_lambda);
        self.total_length += len;
        self.state[0] = v1;
        self.state[1] = v2;
        self.state[2] = v3;
        self.state[3] = v4;
        match leftover {
            Option::Some(leftover) => {
                self.copy_into_internal(leftover);
            }
            _ => {}
        };
    }

    #[inline(always)]
    fn copy_into_internal(&mut self, slice: &[u8]) {
        debug_assert!(self.interior_length < 16);
        debug_assert!(slice.len() <= 16);
        debug_assert!((self.interior_length + slice.len()) <= 16);

        let start = self.interior_length;
        let term = start + slice.len();
        (&mut self.interior_buffer[start..term]).copy_from_slice(slice);

        self.interior_length += slice.len();
        self.total_length += slice.len();

        if self.interior_length == 16 {
            self.state[0] = xxh32_round(
                self.state[0],
                Num::<u32>::read_value_le(&self.interior_buffer[0..4]),
            );
            self.state[1] = xxh32_round(
                self.state[1],
                Num::<u32>::read_value_le(&self.interior_buffer[4..8]),
            );
            self.state[2] = xxh32_round(
                self.state[2],
                Num::<u32>::read_value_le(&self.interior_buffer[8..12]),
            );
            self.state[3] = xxh32_round(
                self.state[3],
                Num::<u32>::read_value_le(&self.interior_buffer[12..16]),
            );
            self.interior_length = 0;
        }
    }

    #[inline]
    fn avaliable(&self) -> usize {
        16 - self.interior_length
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.interior_length == 0
    }
}
impl Hasher for XXHash32 {
    #[inline]
    fn write(&mut self, data: &[u8]) {
        self.consume(data);
    }

    fn finish(&self) -> u64 {
        self.perform_finish() as u64
    }
}

/// This test mostly exists to ensure that our implementation is correct
/// we're using a 3^rd party crate as a reference.
#[test]
fn xxh32_sanity_test() {
    use super::getrandom::getrandom;
    use super::twox_hash::XxHash32;

    let mut seed = [0u8; 4];
    getrandom(seed.as_mut()).unwrap();
    let seed = u32::from_le_bytes(seed);
    let mut core_data = [0u8; 4096];
    getrandom(core_data.as_mut()).unwrap();

    // produce a reference output
    let mut baseline_hasher = XxHash32::with_seed(seed);
    baseline_hasher.write(core_data.as_ref());
    let baseline_output = baseline_hasher.finish() as u32;

    // internal streaming impl
    let mut streaming_hasher = XXHash32::with_seed(seed);
    streaming_hasher.write(core_data.as_ref());
    let streaming_output = streaming_hasher.finish() as u32;

    // internal reference impl
    let reference_output = xxh32_reference(Num::from(seed), core_data.as_ref()).inner();

    assert_eq!(baseline_output, reference_output);
    assert_eq!(streaming_output, reference_output);
    assert_eq!(baseline_output, streaming_output);
}
