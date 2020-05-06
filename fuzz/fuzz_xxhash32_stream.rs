#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate xxhash;

use std::hash::Hasher;

fuzz_target!{
    |data: &[u8]| {
        let mut item = xxhash::bits32::XXHash32::default();
        item.write(data);
        let _ = item.finish();
    }
}
