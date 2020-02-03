use super::feature_macros::intrinsics::hint_likely;
use super::feature_macros::numbers::{Num, PrimativeNumber};
use super::feature_macros::prefetch::prefetch_buffer;

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
        .rotate_left(13)
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
            let ingest = xxh64_round(0, Num::read_value_le(arg));
            (hash ^ ingest)
                .rotate_left(27)
                .wrapping_mul(PRIME64_1)
                .wrapping_mul(PRIME64_4)
        } else {
            arg.chunks(4).fold(hash, fold_chunk_4_wide)
        }
    }
    xxh64_avalanche(buffer.chunks(8).fold(seed, fold_chunk_8_wide))
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
        let mut fold_start: [Num<u64>; 4] = [
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
            (output[0]
                .rotate_left(1)
                .wrapping_add(output[1].rotate_left(7))
                .wrapping_add(output[2].rotate_left(12))
                .wrapping_add(output[3].rotate_left(18))),
            |fold_state, new| xxh64_merge_round(fold_state, new),
        )
    } else {
        seed.wrapping_add(PRIME64_5)
    };
    let last = buffer.len() & (<Num<usize> as PrimativeNumber>::max() - 31usize);
    debug_assert!(last <= buffer.len());
    xxh64_finalize(hash.wrapping_add(buffer.len() as u64), &buffer[last..])
}
