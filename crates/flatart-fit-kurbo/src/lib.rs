//! [`CurveFitter`](flatart_ports::CurveFitter) adapter: fits cubic Béziers to
//! a polyline with kurbo (Pipeline step 7).
//!
//! - `fitter`: the adapter. Corner splitting, the checks and the
//!   fit-or-split loop come from `flatart-fit-support`.
//! - `kurbo_fit`: fits one corner-free run with kurbo.
//!
//! `kurbo` is a private detail of this crate.

mod fitter;
mod kurbo_fit;

pub use fitter::KurboFitter;

#[cfg(test)]
mod tests;
