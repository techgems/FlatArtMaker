//! [`CurveFitter`](flatart_ports::CurveFitter) adapter: fits cubic Béziers to
//! a polyline with kurbo (Pipeline step 7).
//!
//! - `corners`: splits the polyline at turns sharper than the threshold, so
//!   each corner is exactly where one curve ends and the next starts.
//! - `kurbo_fit`: fits one corner-free run with kurbo.
//! - `deviation`: checks a fit against the tolerance both ways. kurbo's own
//!   error measure is approximate, so every fit is checked, and a run that
//!   fails is fitted tighter or split.
//! - `hooks`: rejects fits that double back on themselves the same way.
//!
//! `kurbo` is a private detail of this crate.

mod corners;
mod deviation;
mod fitter;
mod hooks;
mod kurbo_fit;

pub use fitter::KurboFitter;

#[cfg(test)]
mod tests;
