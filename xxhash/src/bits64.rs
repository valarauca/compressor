use super::feature_macros::intrinsics::{hint_likely, memcp};
use super::feature_macros::numbers::{Num, PrimativeNumber};

#[cfg(not(feature = "std"))]
use core::hash::Hasher;
#[cfg(feature = "std")]
use std::hash::Hasher;

#[cfg(not(feature = "std"))]
use core::slice::from_raw_parts;
#[cfg(feature = "std")]
use std::slice::from_raw_parts;

const PRIME64_1: u64 = 11400714785074694791u64;
const PRIME64_2: u64 = 14029467366897019727u64;
const PRIME64_3: u64 = 1609587929392839161u64;
const PRIME64_4: u64 = 9650029242287828579u64;
const PRIME64_5: u64 = 2870177450012600261u64;

#[inline(always)]
fn xxh64_round<A, B>(seed: A, input: B) -> Num<u64>
where
    Num<u64>: From<A>,
    Num<u64>: From<B>,
{
    Num::from(seed)
        .wrapping_add(Num::from(input).wrapping_mul(PRIME64_2))
        .rotate_left(31)
        .wrapping_mul(PRIME64_1)
}

#[inline(always)]
fn xxh64_merge_round<A, B>(seed: A, input: B) -> Num<u64>
where
    Num<u64>: From<A>,
    Num<u64>: From<B>,
{
    (Num::from(seed) ^ xxh64_round(Num::from(0), Num::from(input)))
        .wrapping_mul(PRIME64_1)
        .wrapping_add(PRIME64_4)
}

#[inline(always)]
fn xxh64_avalanche(seed: Num<u64>) -> Num<u64> {
    let mut seed = seed;
    seed ^= seed >> 33;
    seed = seed.wrapping_mul(PRIME64_2);
    seed ^= seed >> 29;
    seed = seed.wrapping_mul(PRIME64_3);
    seed ^= seed >> 32;
    seed
}

#[inline(always)]
fn xxh64_finalize(seed: Num<u64>, buffer: &[u8]) -> Num<u64> {
    /// this function works on the last <3 bytes of the buffer
    #[inline(always)]
    fn inner_most_fold(hash: Num<u64>, arg: &u8) -> Num<u64> {
        let ingest = Num::from((*arg as u64).wrapping_mul(PRIME64_5));
        (hash ^ ingest).rotate_left(11).wrapping_mul(PRIME64_1)
    }

    /// cleans up the 4 byte wide values
    #[inline(always)]
    fn fold_chunk_4_wide(hash: Num<u64>, arg: &[u8]) -> Num<u64> {
        let length = arg.len();
        debug_assert!(length <= 4 && length > 0);
        if length == 4 {
            let ingest =
                Num::from((Num::<u32>::read_value_le(arg).inner() as u64).wrapping_mul(PRIME64_1));
            (hash ^ ingest)
                .rotate_left(23)
                .wrapping_mul(PRIME64_2)
                .wrapping_add(PRIME64_3)
        } else {
            arg.iter().fold(hash, inner_most_fold)
        }
    }

    /// cleans up any 8-byte wide state
    #[inline(always)]
    fn fold_chunk_8_wide(hash: Num<u64>, arg: &[u8]) -> Num<u64> {
        let length = arg.len();
        debug_assert!(length <= 8 && length > 0);
        if length == 8 {
            let ingest = Num::read_value_le(arg)
                .wrapping_mul(PRIME64_2)
                .rotate_left(31)
                .wrapping_mul(PRIME64_1);
            (hash ^ ingest)
                .rotate_left(27)
                .wrapping_mul(PRIME64_1)
                .wrapping_add(PRIME64_4)
        } else {
            arg.chunks(4).fold(hash, fold_chunk_4_wide)
        }
    }

    let hash = if buffer.is_empty() {
        seed
    } else {
        buffer.chunks(8).fold(seed, fold_chunk_8_wide)
    };
    xxh64_avalanche(hash)
}

pub fn xxhash64_reference(seed: u64, buffer: &[u8]) -> u64 {
    xxh64_reference(Num::from(seed), buffer).inner()
}

fn xxh64_reference(seed: Num<u64>, buffer: &[u8]) -> Num<u64> {
    /// this processes our large chunks
    #[inline(always)]
    fn big_chunk(v: [Num<u64>; 4], chunk: &[u8]) -> [Num<u64>; 4] {
        debug_assert!(chunk.len() == 32);
        let n = dereference_32(chunk);
        [
            xxh64_round(v[0], n[0]),
            xxh64_round(v[1], n[1]),
            xxh64_round(v[2], n[2]),
            xxh64_round(v[3], n[3]),
        ]
    }

    let hash = if buffer.len() >= 32 {
        let fold_start: [Num<u64>; 4] = [
            seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2),
            seed.wrapping_add(PRIME64_2),
            seed,
            seed.wrapping_sub(PRIME64_1),
        ];
        let output = buffer
            .chunks(32)
            .filter(|chunk| hint_likely(chunk.len() == 32))
            .fold(fold_start, big_chunk);
        output.iter().fold(
            output[0]
                .rotate_left(1)
                .wrapping_add(output[1].rotate_left(7))
                .wrapping_add(output[2].rotate_left(12))
                .wrapping_add(output[3].rotate_left(18)),
            |fold_state, new| xxh64_merge_round(fold_state, new),
        )
    } else {
        seed.wrapping_add(PRIME64_5)
    };
    let last = buffer.len() & (<Num<usize> as PrimativeNumber>::max() - 31usize);
    debug_assert!(last <= buffer.len());
    xxh64_finalize(hash.wrapping_add(buffer.len() as u64), &buffer[last..])
}

#[derive(Clone)]
pub struct XXHash64 {
    interior_buffer: [u8; 32],
    state: [Num<u64>; 4],
    seed: Num<u64>,
    interior_length: usize,
    total_length: usize,
}
impl Default for XXHash64 {
    fn default() -> XXHash64 {
        XXHash64::new()
    }
}
impl XXHash64 {
    /// Builds an XXHash64 with a seed of `0`, this is identical to `Default`
    pub fn new() -> XXHash64 {
        XXHash64::with_seed(0)
    }

    /// construct with a deterministic seed
    pub fn with_seed(seed: u64) -> XXHash64 {
        let seed = Num::<u64>::from(seed);
        XXHash64 {
            interior_length: 0,
            total_length: 0,
            seed: Num::<u64>::from(seed),
            state: [
                seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2),
                seed.wrapping_add(PRIME64_2),
                seed,
                seed.wrapping_sub(PRIME64_1),
            ],
            interior_buffer: [0u8; 32],
        }
    }

    #[inline]
    fn total_length(&self) -> usize {
        (self.total_length + self.interior_length) as usize
    }

    fn perform_finish(&self) -> u64 {
        debug_assert!(self.interior_length <= 32);

        let hash = if self.total_length() >= 32 {
            let output = self.state[0]
                .rotate_left(1)
                .wrapping_add(self.state[1].rotate_left(7))
                .wrapping_add(self.state[2].rotate_left(12))
                .wrapping_add(self.state[3].rotate_left(18));

            let a = xxh64_merge_round(output, self.state[0]);
            let b = xxh64_merge_round(a, self.state[1]);
            let c = xxh64_merge_round(b, self.state[2]);
            xxh64_merge_round(c, self.state[3])
        } else {
            self.seed.wrapping_add(PRIME64_5)
        };
        let hash = hash.wrapping_add(self.total_length() as u64);
        xxh64_finalize(
            Num::<u64>::from(hash),
            &self.interior_buffer[0..self.interior_length],
        )
        .inner()
    }

    #[inline]
    fn consume<'a>(&mut self, arg: &'a [u8]) {
        let mut arg = arg;

        // check if we need to do slow path stuff
        // to clean up internal state
        if self.interior_length > 0 {
            // short inputs
            if (self.interior_length + arg.len()) < 32 {
                unsafe {
                    memcp(
                        arg.as_ptr(),
                        self.interior_buffer
                            .as_mut_ptr()
                            .offset(self.interior_length as isize),
                        arg.len(),
                    )
                };
                self.interior_length += arg.len();
                // almost immediate return
                return;
            } else {
                // longer inputs
                let chop_off = 32 - self.interior_length;

                // take a window of the remaining data
                arg = unsafe {
                    memcp(
                        arg.as_ptr(),
                        self.interior_buffer
                            .as_mut_ptr()
                            .offset(self.interior_length as isize),
                        chop_off as usize,
                    );
                    from_raw_parts(arg.as_ptr().offset(chop_off as isize), arg.len() - chop_off)
                };

                // cleanup internal state
                self.state[0] = xxh64_round(
                    self.state[0].clone(),
                    Num::<u64>::read_value_le(&self.interior_buffer[0..8]),
                );
                self.state[1] = xxh64_round(
                    self.state[1].clone(),
                    Num::<u64>::read_value_le(&self.interior_buffer[8..16]),
                );
                self.state[2] = xxh64_round(
                    self.state[2].clone(),
                    Num::<u64>::read_value_le(&self.interior_buffer[16..24]),
                );
                self.state[3] = xxh64_round(
                    self.state[3].clone(),
                    Num::<u64>::read_value_le(&self.interior_buffer[24..32]),
                );
                self.total_length += 32;
                self.interior_length = 0;

                // this nows falls throught to the orginal logic
            }
        }

        let last_align = arg.len() & (<Num<usize> as PrimativeNumber>::max() - 31);
        let (nicely_aligned, messy) = arg.split_at(last_align);

        if nicely_aligned.len() > 0 {
            let mut v1 = self.state[0];
            let mut v2 = self.state[1];
            let mut v3 = self.state[2];
            let mut v4 = self.state[3];
            let mut processed = 0;

            for chunk in nicely_aligned.chunks(32) {
                let [n1, n2, n3, n4] = dereference_32(chunk);
                v1 = xxh64_round(v1, n1);
                v2 = xxh64_round(v2, n2);
                v3 = xxh64_round(v3, n3);
                v4 = xxh64_round(v4, n4);
                processed += 1;
            }
            self.total_length += processed * 32;
            self.state[0] = v1;
            self.state[1] = v2;
            self.state[2] = v3;
            self.state[3] = v4;
        }

        // since we cleanuped our internal buffer at the start
        // we can shove what ever remains into the final buffer here
        unsafe {
            memcp(
                messy.as_ptr(),
                self.interior_buffer.as_mut_ptr(),
                messy.len(),
            )
        };
        self.interior_length += messy.len();
    }
}
impl Hasher for XXHash64 {
    #[inline(never)]
    fn write(&mut self, data: &[u8]) {
        self.consume(data);
    }

    fn finish(&self) -> u64 {
        self.perform_finish()
    }
}

/// handles the semantics of a "fast load"
#[inline(always)]
fn dereference_32(arg: &[u8]) -> [u64; 4] {
    #[cfg(not(feature = "std"))]
    use core::ptr::read_unaligned;
    #[cfg(feature = "std")]
    use std::ptr::read_unaligned;

    debug_assert!(arg.len() == 32);

    #[allow(unused_mut)]
    let mut output: [u64; 4] =
        unsafe { read_unaligned::<[u64; 4]>(arg.as_ptr() as *const [u64; 4]) };

    #[cfg(target_endian = "big")]
    {
        for ptr in output.iter_mut() {
            *ptr = ptr.clone().byte_swap();
        }
    }
    output
}

#[cfg(test)]
mod test {

    use super::super::getrandom::getrandom;
    use super::super::twox_hash::XxHash64;
    use super::{xxhash64_reference, XXHash64};

    #[cfg(not(feature = "std"))]
    use core::hash::Hasher;
    #[cfg(feature = "std")]
    use std::hash::Hasher;

    #[test]
    fn xxh64_sanity_test() {
        let mut seed = [0u8; 8];
        getrandom(seed.as_mut()).unwrap();
        let seed = u64::from_le_bytes(seed);

        let mut core_data = [0u8; 4096];
        getrandom(core_data.as_mut()).unwrap();

        let mut baseline_hasher = XxHash64::with_seed(seed);
        baseline_hasher.write(core_data.as_ref());
        let baseline_output = baseline_hasher.finish();

        let mut local_hasher = XXHash64::with_seed(seed);
        local_hasher.write(core_data.as_ref());
        let local_output = local_hasher.finish();

        let reference_output = xxhash64_reference(seed, core_data.as_ref());
        assert_eq!(reference_output, baseline_output);
        assert_eq!(local_output, reference_output);

        // ensure checking doesn't effect the result
        let mut hasher2 = XXHash64::with_seed(seed);
        for chunk in core_data.chunks(4) {
            hasher2.write(chunk);
        }
        assert_eq!(local_output, hasher2.finish());
        let mut hasher3 = XXHash64::with_seed(seed);
        for chunk in core_data.chunks(1) {
            hasher3.write(chunk);
        }
        assert_eq!(local_output, hasher3.finish());
    }

    #[test]
    fn xxh64_empty_test() {
        let seed = 0;
        let dut = &[];
        let expected = 0xEF46DB3751D8E999u64;

        let mut hasher = XXHash64::with_seed(seed);
        hasher.write(dut);
        assert_eq!(expected, hasher.finish());
        assert_eq!(expected, xxhash64_reference(seed, dut));
    }

    #[test]
    fn xxh64_single_byte() {
        let seed = 0;
        let dut = &[42u8];
        let expected = 0x0A9EDECEBEB03AE4u64;

        let mut hasher = XXHash64::with_seed(seed);
        hasher.write(dut);
        assert_eq!(expected, xxhash64_reference(seed, dut));
        assert_eq!(expected, hasher.finish());
    }

    #[test]
    fn xxh64_matches_c() {
        let seed = 0;
        let dut = b"Hello, world!\0";
        assert_eq!(dut.len(), 14);
        let expected = 0x7B06C531EA43E89Fu64;

        let mut hasher = XXHash64::with_seed(seed);
        hasher.write(dut);
        assert_eq!(expected, hasher.finish());
        assert_eq!(expected, xxhash64_reference(seed, dut));
    }

    #[test]
    fn xxh64_matches_c_different_seed() {
        let seed = 0xAE0543311B702d91u64;
        let dut = b"";
        let expected = 0x4B6A04FCDF7A4672u64;

        let mut hasher = XXHash64::with_seed(seed);
        hasher.write(dut);
        assert_eq!(expected, xxhash64_reference(seed, dut));
        assert_eq!(expected, hasher.finish());
    }

    #[test]
    fn xxh64_matches_c_different_seed_and_chunks() {
        let seed = 0xAE0543311B702d91u64;
        let dut = (0..100).collect::<Vec<u8>>();
        let expected = 0x567E355E0682E1F1u64;

        let mut hasher = XXHash64::with_seed(seed);
        hasher.write(&dut);
        assert_eq!(expected, xxhash64_reference(seed, &dut));
        assert_eq!(expected, hasher.finish());
    }
}
