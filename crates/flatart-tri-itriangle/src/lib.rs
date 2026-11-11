//! [`Triangulator`](flatart_ports::Triangulator) adapter: fills a flat
//! polygon with holes using i_triangle (Pipeline step 8).
//!
//! - `rings`: puts each ring the way round i_triangle needs (outer
//!   anticlockwise, holes clockwise) and remembers the input indices.
//! - `quantise`: maps mm to i_triangle's integer grid and back to indices.
//! - `triangulator`: runs it and maps the triangles back onto the input.
//!
//! `i_triangle` is a private detail of this crate.

mod quantise;
mod rings;
mod triangulator;

pub use triangulator::ITriangulator;

#[cfg(test)]
mod tests;
