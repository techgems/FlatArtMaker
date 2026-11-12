//! What every [`CurveFitter`] adapter shares, whichever library fits the
//! curves (Pipeline step 7). An adapter supplies only the function that fits
//! one corner-free run, and calls [`fit`].
//!
//! - `fit`: validates the input, splits at corners, and fits each run,
//!   checking every fit and fitting tighter or splitting when one fails.
//! - `corners`: splits the polyline at turns sharper than the threshold, so
//!   each corner is exactly where one curve ends and the next starts.
//! - `deviation`: checks a fit against the tolerance both ways. Libraries
//!   measure their error approximately, so every fit is checked.
//! - `hooks`: rejects fits that double back on themselves.
//!
//! This crate has no third-party dependencies.
//!
//! [`CurveFitter`]: https://docs.rs/flatart-ports

mod corners;
pub mod deviation;
mod fit;
mod hooks;

pub use fit::fit;
