//! Measuring how far a fit strays from its polyline.
//!
//! Each side is bucketed on a grid as wide as the tolerance, so a point is
//! only measured against the segments near it.

use flatart_domain::{CubicBezier, PxPoint};
use std::collections::HashMap;

/// Points sampled along each curve when checking it.
const SAMPLES_PER_CURVE: usize = 64;

type P = (f64, f64);

fn xy(p: PxPoint) -> P {
    (p.x.0, p.y.0)
}

fn dist_to_segment(p: P, a: P, b: P) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    };
    (p.0 - (a.0 + t * dx)).hypot(p.1 - (a.1 + t * dy))
}

/// A polyline's segments, bucketed by the grid cells they cross.
struct Near<'a> {
    line: &'a [P],
    cell: f64,
    buckets: HashMap<(i64, i64), Vec<usize>>,
}

impl<'a> Near<'a> {
    fn new(line: &'a [P], cell: f64) -> Self {
        let mut buckets: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        let at = |v: f64| (v / cell).floor() as i64;
        for i in 0..line.len().saturating_sub(1) {
            let (a, b) = (line[i], line[i + 1]);
            for x in at(a.0.min(b.0))..=at(a.0.max(b.0)) {
                for y in at(a.1.min(b.1))..=at(a.1.max(b.1)) {
                    buckets.entry((x, y)).or_default().push(i);
                }
            }
        }
        Self {
            line,
            cell,
            buckets,
        }
    }

    /// Whether `p` lies within `tolerance` (at most one cell) of the line.
    fn holds(&self, p: P, tolerance: f64) -> bool {
        if self.line.len() == 1 {
            return dist_to_segment(p, self.line[0], self.line[0]) <= tolerance;
        }
        let at = |v: f64| (v / self.cell).floor() as i64;
        let (cx, cy) = (at(p.0), at(p.1));
        (cx - 1..=cx + 1).any(|x| {
            (cy - 1..=cy + 1).any(|y| {
                self.buckets.get(&(x, y)).is_some_and(|segs| {
                    segs.iter()
                        .any(|&i| dist_to_segment(p, self.line[i], self.line[i + 1]) <= tolerance)
                })
            })
        })
    }
}

/// Whether every point of `curves` lies within `tolerance` of `points`, and
/// every one of `points` within `tolerance` of `curves`.
pub fn within(curves: &[CubicBezier], points: &[PxPoint], tolerance: f64) -> bool {
    let polyline: Vec<P> = points.iter().map(|&p| xy(p)).collect();
    let mut sampled = Vec::with_capacity(curves.len() * SAMPLES_PER_CURVE + 1);
    for (i, curve) in curves.iter().enumerate() {
        let first = usize::from(i > 0);
        sampled.extend(
            (first..=SAMPLES_PER_CURVE)
                .map(|k| xy(curve.point_at(k as f64 / SAMPLES_PER_CURVE as f64))),
        );
    }
    // A cell at least as wide as the tolerance, so the 3 × 3 cells round a
    // point cover everything within it.
    let cell = tolerance.max(1e-6);
    let to_polyline = Near::new(&polyline, cell);
    if !sampled.iter().all(|&q| to_polyline.holds(q, tolerance)) {
        return false;
    }
    let to_curves = Near::new(&sampled, cell);
    polyline.iter().all(|&q| to_curves.holds(q, tolerance))
}
