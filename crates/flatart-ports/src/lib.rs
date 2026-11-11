//! Interfaces between the pipeline (consumer) and adapters (providers).
//!
//! Rules (SPEC.md, "Interface rules"):
//! - Each trait is named for its capability, never for a library.
//! - Signatures use only `flatart-domain` types.
//! - Each trait documents its contract.
//! - Each trait has a conformance suite in `testkit` (feature `testkit`) that every adapter runs.
//!
//! Every method must be deterministic: no I/O, no clocks, seeded randomness.
//! Errors are `flatart_domain::Error`; adapters map their own errors into it.

mod border_tracer;
mod color_quantizer;
mod colour_mapper;
mod curve_fitter;
mod exporter;
mod mesh_builder;
mod raster_stages;
mod slicer_dialect;
mod triangulator;

#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

pub use border_tracer::BorderTracer;
pub use color_quantizer::ColorQuantizer;
pub use colour_mapper::ColourMapper;
pub use curve_fitter::CurveFitter;
pub use exporter::Exporter;
pub use mesh_builder::MeshBuilder;
pub use raster_stages::RasterStages;
pub use slicer_dialect::{MODEL_PATH, SlicerDialect};
pub use triangulator::Triangulator;
