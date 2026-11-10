//! Measuring how far a fit strays from its polyline.

use flatart_domain::{CubicBezier, PxPoint};

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

/// Whether `p` lies within `tolerance` of the polyline `line`.
fn near(p: P, line: &[P], tolerance: f64) -> bool {
    match line {
        [only] => dist_to_segment(p, *only, *only) <= tolerance,
        _ => line
            .windows(2)
            .any(|w| dist_to_segment(p, w[0], w[1]) <= tolerance),
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
    sampled.iter().all(|&q| near(q, &polyline, tolerance))
        && polyline.iter().all(|&q| near(q, &sampled, tolerance))
}
