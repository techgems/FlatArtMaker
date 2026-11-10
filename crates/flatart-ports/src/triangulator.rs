use flatart_domain::{PolygonWithHoles, Result};

/// Fills a flat polygon with holes with triangles (Pipeline step 8).
///
/// # Contract
/// - **Input:** a simple outer ring and simple holes strictly inside it,
///   neither crossing the other. Rings may run either way round.
/// - **Indices:** into `polygon.vertices()` (outer ring, then each hole).
///   No new vertices are added, so neighbouring faces keep identical edges.
/// - **Orientation:** every triangle runs anticlockwise (positive area, y up).
/// - **Coverage:** the triangles cover the polygon exactly once: no triangle
///   is degenerate, and their areas sum to the outer area minus the holes.
/// - **Deterministic.**
pub trait Triangulator {
    fn triangulate(&self, polygon: &PolygonWithHoles) -> Result<Vec<[u32; 3]>>;
}
