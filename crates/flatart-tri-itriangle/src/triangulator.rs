//! Runs i_triangle and maps its triangles back onto the input's indices.

use crate::quantise::{Grid, index_of};
use crate::rings::{Ring, oriented};
use flatart_domain::{Error, MmPoint, PolygonWithHoles, Result, Stage};
use flatart_ports::Triangulator;
use i_triangle::i_overlay::i_float::int::point::IntPoint;
use i_triangle::int::unchecked::IntUncheckedTriangulatable;

/// Triangulates with i_triangle's sweep line, which keeps every input point
/// and adds none.
#[derive(Debug, Clone, Copy, Default)]
pub struct ITriangulator;

fn failed(message: impl Into<String>) -> Error {
    Error::StageFailed {
        stage: Stage::Mesh,
        message: message.into(),
    }
}

fn validate(polygon: &PolygonWithHoles) -> Result<()> {
    let rings = std::iter::once(&polygon.outer).chain(&polygon.holes);
    for ring in rings {
        if ring.len() < 3 {
            return Err(Error::InvalidInput(format!(
                "a ring needs at least 3 points, got {}",
                ring.len()
            )));
        }
        if ring
            .iter()
            .any(|p: &MmPoint| !p.x.0.is_finite() || !p.y.0.is_finite())
        {
            return Err(Error::InvalidInput(
                "a ring has a point that isn't finite".into(),
            ));
        }
    }
    Ok(())
}

impl Triangulator for ITriangulator {
    fn triangulate(&self, polygon: &PolygonWithHoles) -> Result<Vec<[u32; 3]>> {
        validate(polygon)?;
        let rings: Vec<Ring> = oriented(polygon);
        let grid = Grid::covering(&rings);
        let shape: Vec<Vec<IntPoint<i32>>> = rings
            .iter()
            .map(|r| r.points.iter().map(|&p| grid.point(p)).collect())
            .collect();
        let raw = shape.uncheck_triangulate();
        let index = index_of(&grid, &rings);
        // A point where rings touch maps to its first index; both are the
        // same position.
        let input: Vec<u32> = raw
            .points()
            .iter()
            .map(|p| {
                index
                    .get(p)
                    .and_then(|i| i.first().copied())
                    .ok_or_else(|| failed("i_triangle returned a point that isn't in the input"))
            })
            .collect::<Result<_>>()?;
        let flat = raw.triangle_indices::<u32>();
        Ok(flat
            .as_chunks::<3>()
            .0
            .iter()
            .map(|t| t.map(|i| input[i as usize]))
            .collect())
    }
}
