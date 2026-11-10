//! Conformance suites: one per interface, generic over any implementation.
//!
//! Each adapter crate runs its interface's suite from its own tests, e.g.
//!
//! ```ignore
//! #[test]
//! fn conforms() {
//!     flatart_ports::testkit::curve_fitter::run(&KurboFitter::default());
//! }
//! ```
//!
//! A suite panics with a message naming the broken contract clause. A
//! replacement adapter is accepted when it passes the same suite.
//!
//! The provisional interfaces (`RasterStages`, `MeshBuilder`) get their
//! suites once their contracts are finished.

pub mod border_tracer;
pub mod color_quantizer;
pub mod colour_mapper;
pub mod curve_fitter;
pub mod exporter;
pub mod fixtures;
mod geometry;
pub mod mesh;
pub mod slicer_dialect;
pub mod triangulator;

/// Unwraps an adapter result, naming the case that failed.
fn ok<T>(result: flatart_domain::Result<T>, case: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{case}: adapter returned an error: {error}"),
    }
}

#[cfg(test)]
mod dummy_tests;
