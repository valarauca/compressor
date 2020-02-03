use super::feature_macros::intrinsics::hint_likely;
use super::feature_macros::numbers::{Num, PrimativeNumber};
use super::feature_macros::prefetch::prefetch_buffer;

#[cfg(not(feature = "std"))]
use core::hash::Hasher;
#[cfg(feature = "std")]
use std::hash::Hasher;

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
                .rotate_left(31)
                .wrapping_mul(PRIME64_1);
            (hash ^ ingest)
                .rotate_left(27)
                .wrapping_mul(PRIME64_1)
                .wrapping_mul(PRIME64_4)
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
    /// interor round performs the mixin logic
    #[inline(always)]
    fn interior_round(seed: Num<u64>, buffer: &[u8]) -> Num<u64> {
        debug_assert!(buffer.len() == 8);
        xxh64_round(seed, Num::read_value_le(buffer))
    }

    /// this processes our large chunks
    #[inline(always)]
    fn big_chunk(v: [Num<u64>; 4], chunk: &[u8]) -> [Num<u64>; 4] {
        debug_assert!(chunk.len() == 32);
        [
            interior_round(v[0], &chunk[0..8]),
            interior_round(v[1], &chunk[8..16]),
            interior_round(v[2], &chunk[16..24]),
            interior_round(v[3], &chunk[24..32]),
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
    interior_length: usize,
    total_length: usize,
    seed: Num<u64>,
    state: [Num<u64>; 4],
    interior_buffer: [u8; 32],
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

    fn perform_finish(&self) -> u64 {
        debug_assert!(self.interior_length <= 32);

        let hash = if self.total_length >= 32 {
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
        let hash = hash.wrapping_add(self.total_length as u64);
        xxh64_finalize(
            Num::<u64>::from(hash),
            &self.interior_buffer[0..self.interior_length],
        )
        .inner()
    }

    /// all the conditional branching.
    #[inline(always)]
    fn maybe_consume<'a>(&mut self, slice: &'a [u8]) -> Option<&'a [u8]> {
        if self.is_empty() {
            Some(slice)
        } else {
            if slice.len() <= self.avaliable() {
                self.copy_into_internal(slice);
                None
            } else {
                let (flush_internal, remaining) = slice.split_at(self.avaliable());
                self.copy_into_internal(flush_internal);
                if remaining.is_empty() {
                    None
                } else {
                    Some(remaining)
                }
            }
        }
    }

    #[inline]
    fn consume<'a>(&mut self, arg: &'a [u8]) {
        for process in self.maybe_consume(arg) {
            for chunk in process.chunks(32) {
                if hint_likely(chunk.len() == 32) {
                    for (arr,state) in chunk.chunks(8).zip(self.state.iter_mut()) {
                        *state = xxh64_round(*state, Num::<u64>::read_value_le(arr));
                    }
                    self.total_length += 32;
                } else {
                    self.copy_into_internal(chunk);
                }
            }
        }
    }

    /// handle messy small writes to the internal buffer.
    #[inline(always)]
    fn copy_into_internal(&mut self, slice: &[u8]) {
        debug_assert!(self.interior_length < 32);
        debug_assert!(slice.len() <= 32);
        debug_assert!((self.interior_length + slice.len()) <= 32);

        if slice.is_empty() {
            return;
        }

        let start = self.interior_length;
        let term = start + slice.len();
        (&mut self.interior_buffer[start..term]).copy_from_slice(slice);

        self.interior_length += slice.len();
        self.total_length += slice.len();

        if self.interior_length == 32 {
            self.state[0] = xxh64_round(
                self.state[0],
                Num::<u64>::read_value_le(&self.interior_buffer[0..8]),
            );
            self.state[1] = xxh64_round(
                self.state[1],
                Num::<u64>::read_value_le(&self.interior_buffer[8..16]),
            );
            self.state[2] = xxh64_round(
                self.state[2],
                Num::<u64>::read_value_le(&self.interior_buffer[16..24]),
            );
            self.state[3] = xxh64_round(
                self.state[3],
                Num::<u64>::read_value_le(&self.interior_buffer[12..32]),
            );
            self.interior_length = 0;
        }
    }

    #[inline]
    fn avaliable(&self) -> usize {
        32 - self.interior_length
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.interior_length == 0
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

#[test]
fn xxh64_sanity_test() {
    use super::getrandom::getrandom;
    use super::twox_hash::XxHash64;

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

    let reference_output = xxh64_reference(Num::from(seed), core_data.as_ref()).inner();
    assert_eq!(reference_output, baseline_output);
    assert_eq!(local_output, reference_output);
}
