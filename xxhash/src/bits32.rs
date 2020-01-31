use super::feature_macros::intrinsics::{hint_likely, hint_unlikely};
use super::feature_macros::numbers::{Num, PrimativeNumber};
use super::feature_macros::prefetch::prefetch_buffer;
use core::hash::Hasher;
use core::iter::{FlatMap, Zip};
use core::ptr::copy_nonoverlapping;
use core::slice::{from_raw_parts, Chunks, IterMut};

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
        assert!(
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
        assert!(buffer.len() == 4, "buffer should always be 4 elements");
        xxh32_round(seed, Num::<u32>::read_value_le(buffer))
    }

    // this is an uglier lambda that is used to consume 16 bytes
    // at a time.
    #[inline(always)]
    fn big_chunk(v: [Num<u32>; 4], chunk: &[u8]) -> [Num<u32>; 4] {
        assert!(chunk.len() == 16);
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
    let last = buffer.len() & (::core::usize::MAX - 15usize);
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
    #[allow(unused_imports)]
    use core::slice::from_raw_parts;
    xxh32_reference(Num::from(seed), from_raw_parts(ptr, len)).inner()
}

/// XXHash32 is an implementation of two-x hash.
/// this structure offers a streaming implementation
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

    /// for streaming operations we need to do this 1 byte at a time
    fn append_byte(&mut self, byte: u8) {
        // if we append the last byte, we'll process
        assert!(self.interior_length <= 15);

        self.interior_buffer[self.interior_length] = byte;

        // update counters
        self.interior_length += 1;
        self.total_length += 1;

        // conditionally flush state
        if self.interior_length == 16 {
            self.preform_round();
        }
    }

    fn perform_finish(&mut self) -> u32 {
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
        assert!(self.interior_length <= 16);
        xxh32_finalize(
            Num::<u32>::from(hash),
            &self.interior_buffer[0..self.interior_length],
        )
        .inner()
    }

    /// consumes the `interior_buffer` worth of data.
    fn preform_round(&mut self) {
        if hint_unlikely(self.interior_length != 16) {
            inconceivable!("perform_round should only be called when internal buffer is full");
        }

        // preform our round
        for (state, chunk) in self.state.iter_mut().zip(self.interior_buffer.chunks(4)) {
            *state = mixin_bytes(*state, chunk);
        }

        // clean up the interior state
        self.interior_length = 0;
        self.interior_buffer = [0u8; 16];
    }

    /// explict borrowing methods
    #[inline(always)]
    fn state_borrow<'a>(&'a mut self) -> IterMut<'a, Num<u32>> {
        self.state.iter_mut()
    }

    /// another explit borrowing method
    #[inline(always)]
    fn state_merge_16<'a, 'b>(
        &'b mut self,
        arg: &'a [u8],
    ) -> Zip<Chunks<'a, u8>, IterMut<'b, Num<u32>>> {
        if hint_unlikely(arg.len() != 16) {
            inconceivable!("state_merge input should always be 16 bytes long");
        }
        arg.chunks(4).zip(self.state_borrow())
    }

    /// another explicit borrowing method
    #[inline(always)]
    fn state_merge_mod_16<'a, 'b>(
        &'b mut self,
        arg: &'a [u8],
    ) -> FlatMap<
        Chunks<'a, u8>,
        Zip<Chunks<'a, u8>, IterMut<'b, Num<u32>>>,
        <XXHash32 as Trait>::state_merge_16,
    > {
        arg.chunks(16).flat_map(|chunk| self.state_merge_16(chunk))
    }
}
impl Hasher for XXHash32 {
    fn write(&mut self, data: &[u8]) {
        if hint_unlikely(self.interior_length >= 16) {
            inconceivable!("this function cannot be called when a round should be performed");
        }

        // case 1: virgin internal state & nice aligned data
        //         this case is overly optimized as it doesnt
        //         require mutating internal state.
        if hint_unlikely(
            (self.interior_length == 0) && (data.len() > 16) && ((data.len() & 0x0Fusize) == 0),
        ) {
            if hint_unlikely(data.len() <= 16) {
                inconceivable!("this should only be reachable by values >= 16");
            }
            if hint_unlikely((data.len() % 16) != 0) {
                inconceivable!("data should be nicely divisable by 16");
            }
            for (chunk, state) in data
                .chunks(16)
                .flat_map(|chunk| chunk.chunks(4).zip(self.state.iter_mut()))
            {
                if hint_unlikely(chunk.len() != 4) {
                    inconceivable!("input is nicely divisable by 16, so WTF?");
                }
                *state = mixin_bytes(*state, chunk);
            }
            self.total_length += data.len();
            // as our input is cleanly divisable by 16
            // we do not need to update `interior_length`
            return;
        }
        unreachable!()
    }

    fn finish(&mut self) -> u64 {
        self.perform_finish() as u64
    }
}

#[inline(always)]
fn mix_in(seed: Num<u32>, input: Num<u32>) -> Num<u32> {
    seed.wrapping_add(input.wrapping_mul(PRIME32_2))
        .rotate_left(13)
        .wrapping_mul(PRIME32_1)
}

#[inline(always)]
fn mixin_bytes(seed: Num<u32>, buffer: &[u8]) -> Num<u32> {
    mix_in(seed, Num::<u32>::read_value_le(buffer))
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

    let reference_output = xxh32_reference(Num::from(seed), core_data.as_ref()).inner();

    assert_eq!(baseline_output, reference_output);
}
