//! The integer grid i_triangle works on.

use crate::rings::Ring;
use i_triangle::i_overlay::i_float::int::point::IntPoint;
use std::collections::HashMap;

/// Half the grid's span: coordinates run from −2²⁷ to 2²⁷, so i_triangle's
/// products stay well inside its 64-bit arithmetic. Across 100 mm that's a
/// step of under a millionth of a millimetre.
const HALF_SPAN: f64 = (1u64 << 27) as f64;

pub struct Grid {
    centre: (f64, f64),
    scale: f64,
}

impl Grid {
    /// A grid covering every point of `rings`.
    pub fn covering(rings: &[Ring]) -> Self {
        let points = rings.iter().flat_map(|r| r.points.iter());
        let (mut lo, mut hi) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
        for &(x, y) in points {
            lo = (lo.0.min(x), lo.1.min(y));
            hi = (hi.0.max(x), hi.1.max(y));
        }
        let half = ((hi.0 - lo.0).max(hi.1 - lo.1) / 2.0).max(f64::MIN_POSITIVE);
        Self {
            centre: ((lo.0 + hi.0) / 2.0, (lo.1 + hi.1) / 2.0),
            scale: HALF_SPAN / half,
        }
    }

    pub fn point(&self, (x, y): (f64, f64)) -> IntPoint<i32> {
        let q = |v: f64, c: f64| ((v - c) * self.scale).round() as i32;
        IntPoint::new(q(x, self.centre.0), q(y, self.centre.1))
    }
}

/// Every input index at each grid point. A point appears more than once
/// where rings touch.
pub fn index_of(grid: &Grid, rings: &[Ring]) -> HashMap<IntPoint<i32>, Vec<u32>> {
    let mut map: HashMap<IntPoint<i32>, Vec<u32>> = HashMap::new();
    for ring in rings {
        for (&p, &i) in ring.points.iter().zip(&ring.indices) {
            map.entry(grid.point(p)).or_default().push(i);
        }
    }
    map
}
