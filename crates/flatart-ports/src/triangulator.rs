use flatart_domain::{PolygonWithHoles, Result};

/// Fills a flat polygon with holes with triangles (Pipeline step 8).
///
/// # Contract
/// - **Input:** an outer ring and holes inside it, at least 3 points each.
///   No ring crosses itself or another, but rings may touch at a point they
///   both list (a traced area can meet itself at a pixel corner). Rings may
///   run either way round.
/// - **Indices:** into `polygon.vertices()` (outer ring, then each hole).
///   No new vertices are added, so neighbouring faces keep identical edges.
///   Where rings touch, any of the indices at that point may be used.
/// - **Edges kept:** every edge of every ring is a side of exactly one
///   triangle, so every point is used and walls built on the rings meet the
///   triangles exactly.
/// - **Orientation:** every triangle runs anticlockwise (positive area, y up).
/// - **Coverage:** the triangles cover the polygon exactly once: no triangle
///   is degenerate, and their areas sum to the outer area minus the holes.
/// - **Deterministic.**
pub trait Triangulator {
    fn triangulate(&self, polygon: &PolygonWithHoles) -> Result<Vec<[u32; 3]>>;
}
