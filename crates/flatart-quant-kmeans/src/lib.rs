//! [`ColorQuantizer`](flatart_ports::ColorQuantizer) adapter: weighted
//! k-means in OKLab (Pipeline step 2).
//!
//! - Each distinct opaque colour is one point, weighted by its pixel count,
//!   so identical pixels always share a label and the work scales with the
//!   number of distinct colours, not pixels.
//! - Pinned colours are fixed centres that k-means never moves.
//! - Randomness is seeded, so the same input always gives the same output.
//!
//! `palette` (sRGB → OKLab) is a private detail of this crate.

mod histogram;
mod kmeans;
mod lab;
mod quantizer;

pub use quantizer::KmeansQuantizer;

#[cfg(test)]
mod tests;
