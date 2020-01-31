extern crate rustc_version;
use rustc_version::{version_meta, Channel};

fn main() {
    let version_info = match version_meta() {
        Ok(v) => v,
        Err(e) => panic!("could not identify rustc version. error:'{:?}'", e),
    };

    if version_info.semver.major != 1 {
        panic!("presently we only support version 1.X of rustc");
    }

    match version_info.channel {
        Channel::Dev => {
            println!(r#"cargo:rustc-cfg=feature="RUSTC_DEV""#);
        }
        Channel::Nightly => {
            println!(r#"cargo:rustc-cfg=feature="RUSTC_NIGHTLY""#);
        }
        Channel::Beta => {
            println!(r#"cargo:rustc-cfg=feature="RUSTC_BETA""#);
        }
        Channel::Stable => {
            println!(r#"cargo:rustc-cfg=feature="RUSTC_STABLE""#);
        }
    };

    if version_info.semver.major == 1 && version_info.semver.minor >= 27 {
        // special configuration variable for compiler version 1.27
        // this is when `core::hint::unchecked_unreachable` was
        // stablized.
        println!("cargo:rustc-cfg=RUSTC_VERSION_GE_1_27");
    }

    if version_info.semver.major == 1 && version_info.semver.minor >= 27 {
        // special configuration variable for compiler version 1.26
        // this is when `u128` was
        // stablized.
        println!("cargo:rustc-cfg=RUSTC_VERSION_GE_1_26");
    }
}
