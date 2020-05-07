#[cfg(feature = "std")]
pub use std::ops::*;

#[cfg(not(feature = "std"))]
pub use core::ops::*;
