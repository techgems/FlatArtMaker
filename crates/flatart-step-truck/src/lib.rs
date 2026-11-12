//! [`Exporter<VectorModel>`](flatart_ports::Exporter) for STEP (AP214), the
//! file people edit in CAD (Pipeline step 9).
//!
//! - `place`: the borders' Béziers in model space (mm, y up), placed like
//!   the 3MF.
//! - `loops`: the boundary of a set of areas as closed loops of borders, and
//!   which loops are holes in which.
//! - `parts`: the base and each colour as slabs of areas.
//! - `chain`: joins each run of tangent curves into one B-spline edge.
//! - `brep`: one closed solid per area of a slab, with truck: flat top and
//!   bottom, vertical walls, every curve exact.
//! - `write`: one component per colour (plus `Base`), then the fixes truck's
//!   writer needs (schema, body names).
//! - `compact`: writes each repeated point and direction once.
//! - `exporter`: the adapter.
//! - `check` (feature `check`, for tests): reads a file back and checks it.
//!
//! `truck` is a private detail of this crate.

mod brep;
mod chain;
#[cfg(feature = "check")]
pub mod check;
mod compact;
mod exporter;
mod loops;
mod parts;
mod place;
mod write;

pub use exporter::StepExporter;

#[cfg(test)]
mod tests;
