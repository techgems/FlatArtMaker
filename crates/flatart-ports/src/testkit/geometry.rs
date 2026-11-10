//! Plain-f64 geometry for the suites' checks.

use flatart_domain::{CubicBezier, MmPoint, PxPoint};

pub type P = (f64, f64);

pub fn px(p: PxPoint) -> P {
    (p.x.0, p.y.0)
}

pub fn mm(p: MmPoint) -> P {
    (p.x.0, p.y.0)
}

pub fn dist(a: P, b: P) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// Distance from `p` to the segment `a`–`b`.
pub fn dist_to_segment(p: P, a: P, b: P) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        return dist(p, a);
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0);
    dist(p, (a.0 + t * dx, a.1 + t * dy))
}

/// Distance from `p` to a polyline.
pub fn dist_to_polyline(p: P, line: &[P]) -> f64 {
    match line {
        [] => f64::INFINITY,
        [only] => dist(p, *only),
        _ => line
            .windows(2)
            .map(|w| dist_to_segment(p, w[0], w[1]))
            .fold(f64::INFINITY, f64::min),
    }
}

/// `per_curve` + 1 points along each curve, joined into one polyline.
pub fn sample(curves: &[CubicBezier], per_curve: usize) -> Vec<P> {
    let mut points = Vec::new();
    for (i, curve) in curves.iter().enumerate() {
        let first = if i == 0 { 0 } else { 1 };
        for k in first..=per_curve {
            points.push(px(curve.point_at(k as f64 / per_curve as f64)));
        }
    }
    points
}

fn cross(o: P, a: P, b: P) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

/// True when the segments cross at a point inside both (touching ends don't count).
pub fn segments_cross(a: P, b: P, c: P, d: P) -> bool {
    let eps = 1e-9;
    let (d1, d2) = (cross(c, d, a), cross(c, d, b));
    let (d3, d4) = (cross(a, b, c), cross(a, b, d));
    ((d1 > eps && d2 < -eps) || (d1 < -eps && d2 > eps))
        && ((d3 > eps && d4 < -eps) || (d3 < -eps && d4 > eps))
}

/// Signed area: positive when the ring runs anticlockwise with y up.
pub fn signed_area(ring: &[P]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum::<f64>()
        / 2.0
}

/// Even-odd point-in-polygon test.
pub fn inside(p: P, ring: &[P]) -> bool {
    let n = ring.len();
    let mut result = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + n - 1) % n]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            result = !result;
        }
    }
    result
}
