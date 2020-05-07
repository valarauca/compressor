#[cfg(feature = "std")]
pub use std::borrow::*;

#[cfg(not(feature = "std"))]
pub use core::borrow::*;
