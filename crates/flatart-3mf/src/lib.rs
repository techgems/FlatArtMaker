//! [`Exporter<MeshModel>`](flatart_ports::Exporter) for 3MF, plus the
//! [`SlicerDialect`](flatart_ports::SlicerDialect) adapters `prusa` and
//! `bambu` (Pipeline step 9), following `spikes/3mf/FINDINGS.md`.
//!
//! - `xml`: shared XML writing (private).
//! - `model`: the shared pieces of `3D/3dmodel.model`: the colour group,
//!   meshes and where the object sits.
//! - `prusa`: one object, one mesh, triangle ranges per part.
//! - `bambu`: one mesh object per part under a parent object.
//! - `package`: the exporter: `[Content_Types].xml`, `_rels/`, the
//!   thumbnail and the zip around a dialect's files.
//!
//! `zip`, `quick-xml` and `png` are private details of this crate.

mod bambu;
mod model;
mod package;
mod prusa;
mod xml;

pub use bambu::BambuDialect;
pub use package::ThreeMfExporter;
pub use prusa::PrusaDialect;

#[cfg(test)]
mod tests;
