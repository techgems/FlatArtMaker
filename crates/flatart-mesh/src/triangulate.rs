//! Filling each region through the `Triangulator`.

use crate::flatten::Outline;
use crate::regions::{Region, RingPoint};
use flatart_domain::{MmPoint, PolygonWithHoles, Result};
use flatart_ports::Triangulator;
use std::collections::HashMap;
use std::f64::consts::TAU;

/// The clockwise angle from direction `a` round to direction `b`, 0 to 2π.
fn clockwise(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.1.atan2(a.0) - b.1.atan2(b.0)).rem_euclid(TAU)
}

/// Where the region's ring passes a point: the directions to the points
/// before and after it.
struct Pass {
    ring_point: RingPoint,
    to_prev: (f64, f64),
    to_next: (f64, f64),
}

impl Pass {
    /// Whether direction `d` from the point lies in the region here. Rings
    /// keep the region on their right (clockwise side, y up), so the region
    /// spans clockwise from the next point round to the previous one.
    fn holds(&self, d: (f64, f64)) -> bool {
        clockwise(self.to_next, d) < clockwise(self.to_next, self.to_prev)
    }
}

/// The region's triangles, anticlockwise (y up), as ring points.
///
/// Where the region meets itself at a point, the triangulator may give any
/// of the ring's passes there; each triangle gets the pass whose side of the
/// point it lies on.
pub fn triangulate(
    outline: &Outline,
    region: &Region,
    triangulator: &dyn Triangulator,
) -> Result<Vec<[RingPoint; 3]>> {
    let at = |r: &RingPoint| outline.points[r.point as usize];
    let mm = |ring: &[RingPoint]| -> Vec<MmPoint> {
        ring.iter()
            .map(|r| {
                let (x, y) = at(r);
                MmPoint::new(x, y)
            })
            .collect()
    };
    let polygon = PolygonWithHoles {
        outer: mm(&region.rings[0]),
        holes: region.rings[1..].iter().map(|r| mm(r)).collect(),
    };
    let flat: Vec<RingPoint> = region.rings.iter().flatten().copied().collect();

    let mut passes: HashMap<u32, Vec<Pass>> = HashMap::new();
    for ring in &region.rings {
        let n = ring.len();
        for i in 0..n {
            let p = at(&ring[i]);
            let (prev, next) = (at(&ring[(i + n - 1) % n]), at(&ring[(i + 1) % n]));
            passes.entry(ring[i].point).or_default().push(Pass {
                ring_point: ring[i],
                to_prev: (prev.0 - p.0, prev.1 - p.1),
                to_next: (next.0 - p.0, next.1 - p.1),
            });
        }
    }

    let triangles = triangulator.triangulate(&polygon)?;
    Ok(triangles
        .into_iter()
        .map(|tri| {
            let corners = tri.map(|i| flat[i as usize]);
            let pts = corners.map(|c| at(&c));
            let centroid = (
                (pts[0].0 + pts[1].0 + pts[2].0) / 3.0,
                (pts[0].1 + pts[1].1 + pts[2].1) / 3.0,
            );
            corners.map(|c| match passes.get(&c.point) {
                Some(list) if list.len() > 1 => {
                    let p = at(&c);
                    let d = (centroid.0 - p.0, centroid.1 - p.1);
                    list.iter()
                        .find(|pass| pass.holds(d))
                        .map_or(c, |pass| pass.ring_point)
                }
                _ => c,
            })
        })
        .collect())
}
