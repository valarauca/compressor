#[cfg(feature = "std")]
pub use std::hash::*;

#[cfg(not(feature = "std"))]
pub use core::hash::*;
