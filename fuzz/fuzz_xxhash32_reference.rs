#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate xxhash;

fuzz_target!{
    |data: &[u8]| {
        let _ = xxhash::bits32::xxhash32_reference(0, data);
    }
}
