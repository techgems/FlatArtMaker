//! [`MeshBuilder`](flatart_ports::MeshBuilder) implementation: extrudes the
//! shared-border map into watertight parts (Pipeline step 8).
//!
//! - `flatten`: each border into one polyline in mm, once.
//! - `regions`: the areas to fill, as rings of those points: each face,
//!   plus the outside areas the art encloses (for filling the base).
//! - `triangulate`: fills each region through the injected `Triangulator`.
//! - `solid`: builds one closed part from regions and their heights.
//! - `builder`: which parts there are and how high each region goes.
//!
//! No geometry library is used here: triangulation goes only through the port.

mod builder;
mod flatten;
mod regions;
mod solid;
mod triangulate;

pub use builder::Extruder;

#[cfg(test)]
mod tests;
