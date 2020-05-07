#[cfg(feature = "std")]
pub use std::ptr::*;

#[cfg(not(feature = "std"))]
pub use core::ptr::*;
