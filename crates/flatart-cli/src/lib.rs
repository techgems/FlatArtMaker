//! The native wiring: the only place (with the WASM wiring) that names the
//! concrete adapters.
//!
//! - `wiring`: builds a [`Pipeline`] with every adapter.
//! - `job`: what to make, and a pipeline set up for it.
//! - `png_image`: reads a PNG into an `RgbaImage`.
//!
//! `main.rs` parses the command line and writes the files.

mod job;
mod png_image;
mod wiring;

pub use job::{Fitter, Job, Slicer, ThinFix};
pub use png_image::read_png;
pub use wiring::pipeline;

pub use flatart_pipeline::Pipeline;
