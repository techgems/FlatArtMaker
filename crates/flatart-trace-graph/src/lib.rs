//! [`BorderTracer`](flatart_ports::BorderTracer) implementation: builds the
//! shared-border map from a label map (Pipeline step 7).
//!
//! - `grid`: cells, the vertex lattice between them, and the pixel edges
//!   where two labels differ (beyond the image counts as outside).
//! - `chains`: walks those edges into borders, split at junctions.
//! - `simplify`: turns each border's pixel staircase into the points to fit.
//! - `curves`: fits every border through the injected `CurveFitter`, and
//!   refits more tightly any that would cross.
//! - `crossings`: finds borders whose curves cross.
//! - `faces`: assembles each 4-connected area's loops from the borders.
//!
//! No curve library is used here: fitting goes only through the port.

mod chains;
mod crossings;
mod curves;
mod faces;
mod grid;
mod simplify;
mod tracer;

pub use tracer::GraphTracer;

#[cfg(test)]
mod tests;
