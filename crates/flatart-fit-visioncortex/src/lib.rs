//! [`CurveFitter`](flatart_ports::CurveFitter) adapter: fits cubic Béziers to
//! a polyline with visioncortex, the curve fitting behind vtracer (Pipeline
//! step 7).
//!
//! - `fitter`: the adapter. Corner splitting, the checks and the
//!   fit-or-split loop come from `flatart-fit-support`.
//! - `vc_fit`: fits one corner-free run with visioncortex.
//!
//! `visioncortex` is a private detail of this crate.

mod fitter;
mod vc_fit;

pub use fitter::VisionCortexFitter;

#[cfg(test)]
mod tests;
