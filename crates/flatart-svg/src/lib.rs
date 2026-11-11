//! [`Exporter<VectorModel>`](flatart_ports::Exporter) for SVG (Pipeline
//! step 9).
//!
//! - `place`: where the art sits, in mm.
//! - `path`: one face as path data, from the shared Béziers.
//! - `document`: the file: a layer per output colour, a path per face.
//!
//! A plain string writer: no dependencies.

mod document;
mod exporter;
mod path;
mod place;

pub use exporter::SvgExporter;

#[cfg(test)]
mod tests;
