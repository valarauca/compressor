#[cfg(feature = "std")]
pub use std::mem::*;

#[cfg(not(feature = "std"))]
pub use core::mem::*;
