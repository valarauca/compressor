use super::feature_macros::intrinsics::hint_likely;
use super::feature_macros::numbers::{Num, PrimativeNumber};
use super::feature_macros::prefetch::prefetch_buffer;

#[cfg(not(feature = "std"))]
use core::hash::Hasher;
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
    /// this works on the last <3 bytes of the buffer
    #[inline(always)]
    fn inner_most_fold(hash: Num<u32>, arg: &u8) -> Num<u32> {
        let ingest = (*arg as u32).wrapping_mul(PRIME32_5);
        hash.wrapping_add(ingest)
            .rotate_left(11)
            .wrapping_mul(PRIME32_1)
    }

    /// this is invoked on the last few 4 byte wide chunks
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
    debug_assert!(last <= buffer.len());
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

    /// all the conditional branching.
    #[inline(always)]
    fn maybe_consume<'a>(&mut self, slice: &'a [u8]) -> Option<&'a [u8]> {
        if !self.is_empty() {
            if slice.len() < self.avaliable() {
                self.copy_into_internal(slice);
                None
            } else {
                let (flush_internal, remaining) = slice.split_at(self.avaliable());
                self.copy_into_internal(flush_internal);
                debug_assert!(self.is_empty());
                if remaining.is_empty() {
                    None
                } else {
                    Some(remaining)
                }
            }
        } else {
            Some(slice)
        }
    }

    #[inline]
    fn consume<'a>(&mut self, arg: &'a [u8]) {
        for remaining_data in self.maybe_consume(arg) {
            for chunk in remaining_data.chunks(16) {
                if chunk.len() == 16 {
                    self.state[0] =
                        xxh32_round(self.state[0], Num::<u32>::read_value_le(&chunk[0..4]));
                    self.state[1] =
                        xxh32_round(self.state[1], Num::<u32>::read_value_le(&chunk[4..8]));
                    self.state[2] =
                        xxh32_round(self.state[2], Num::<u32>::read_value_le(&chunk[8..12]));
                    self.state[3] =
                        xxh32_round(self.state[3], Num::<u32>::read_value_le(&chunk[12..16]));
                    self.total_length += 16;
                } else {
                    self.copy_into_internal(chunk);
                }
            }
        }
    }

    /// handle messy small writes to the internal buffer.
    #[inline(always)]
    fn copy_into_internal(&mut self, slice: &[u8]) {
        debug_assert!(self.interior_length < 16);
        debug_assert!(slice.len() <= 16);
        debug_assert!((self.interior_length + slice.len()) <= 16);

        if slice.is_empty() {
            return;
        }

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
    #[inline(never)]
    fn write(&mut self, data: &[u8]) {
        if !self.is_empty() {}
        self.consume(data);
    }

    fn finish(&self) -> u64 {
        self.perform_finish() as u64
    }
}

#[cfg(test)]
mod test {

    use super::super::getrandom::getrandom;
    use super::super::twox_hash::XxHash32;
    use super::{xxhash32_reference, XXHash32};

    #[cfg(not(feature = "std"))]
    use core::hash::Hasher;
    #[cfg(feature = "std")]
    use std::hash::Hasher;

    /// This test mostly exists to ensure that our implementation is correct
    /// we're using a 3^rd party crate as a reference.
    #[test]
    fn xxh32_sanity_test() {
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
        let reference_output = xxhash32_reference(seed, core_data.as_ref());

        assert_eq!(baseline_output, reference_output);
        assert_eq!(streaming_output, reference_output);
        assert_eq!(baseline_output, streaming_output);
    }

    /*
     * credit to: Jake Goulding & Collaborators of twox_hash, copyright
     * unchallenged.
     *
     */

    #[test]
    fn xxh32_matches_c_for_empty_inputs() {
        let seed = 0;
        let dut = &[];
        let expected = 0x02CC5D05u32;

        let mut hasher = XXHash32::with_seed(seed);
        hasher.write(dut);
        assert_eq!(expected, hasher.finish() as u32);
        assert_eq!(expected, xxhash32_reference(seed, dut));
    }

    #[test]
    fn xxh32_matches_c_for_single_byte() {
        let seed = 0;
        let dut = &[42u8];
        let expected = 0xE0FE705Fu32;

        let mut hasher = XXHash32::with_seed(seed);
        hasher.write(dut);
        assert_eq!(expected, hasher.finish() as u32);
        assert_eq!(expected, xxhash32_reference(seed, dut));
    }

    #[test]
    fn xxh32_matches_multiple_bytes() {
        let seed = 0;
        let dut = b"Hello, world!\0";
        let expected = 0x9E5E7E93u32;

        let mut hasher = XXHash32::with_seed(seed);
        hasher.write(dut);
        assert_eq!(expected, hasher.finish() as u32);
        assert_eq!(expected, xxhash32_reference(seed, dut));
    }

    #[test]
    fn xxh32_matches_chunks() {
        let seed = 0;
        let dut = (0..100).collect::<Vec<u8>>();
        let expected = 0x7F89BA44u32;

        let mut hasher = XXHash32::with_seed(seed);
        hasher.write(&dut);
        assert_eq!(expected, hasher.finish() as u32);
        assert_eq!(expected, xxhash32_reference(seed, &dut));
    }

    #[test]
    fn xxh32_matches_empty_with_different_seed() {
        let seed = 0x42C91977u32;
        let dut = b"";
        let expected = 0xD6BF8459u32;

        let mut hasher = XXHash32::with_seed(seed);
        hasher.write(dut);
        assert_eq!(expected, hasher.finish() as u32);
        assert_eq!(expected, xxhash32_reference(seed, dut));
    }

    #[test]
    fn xxh32_matches_chunks_with_different_seed() {
        let seed = 0x42C91977u32;
        let dut = (0..100).collect::<Vec<u8>>();
        let expected = 0x6D2F6C17u32;

        let mut hasher = XXHash32::with_seed(seed);
        hasher.write(&dut);
        assert_eq!(expected, hasher.finish() as u32);
        assert_eq!(expected, xxhash32_reference(seed, &dut));
    }
}
