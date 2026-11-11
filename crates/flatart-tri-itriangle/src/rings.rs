//! Rings the way round i_triangle needs them.

use flatart_domain::{MmPoint, PolygonWithHoles};

/// One ring, in the order i_triangle needs, with each point's index in
/// `PolygonWithHoles::vertices`.
pub struct Ring {
    pub points: Vec<(f64, f64)>,
    pub indices: Vec<u32>,
}

fn signed_area(ring: &[MmPoint]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            a.x.0 * b.y.0 - b.x.0 * a.y.0
        })
        .sum::<f64>()
        / 2.0
}

fn ring(points: &[MmPoint], first: u32, anticlockwise: bool) -> Ring {
    let mut ring = Ring {
        points: points.iter().map(|p| (p.x.0, p.y.0)).collect(),
        indices: (first..first + points.len() as u32).collect(),
    };
    if (signed_area(points) > 0.0) != anticlockwise {
        ring.points.reverse();
        ring.indices.reverse();
    }
    ring
}

/// The outer ring anticlockwise, then each hole clockwise.
pub fn oriented(polygon: &PolygonWithHoles) -> Vec<Ring> {
    let mut first = polygon.outer.len() as u32;
    let mut rings = vec![ring(&polygon.outer, 0, true)];
    for hole in &polygon.holes {
        rings.push(ring(hole, first, false));
        first += hole.len() as u32;
    }
    rings
}
