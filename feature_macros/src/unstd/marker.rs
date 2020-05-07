#[cfg(feature = "std")]
pub use std::marker::*;

#[cfg(not(feature = "std"))]
pub use core::marker::*;
