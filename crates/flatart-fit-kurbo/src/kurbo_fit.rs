//! One corner-free run, fitted by kurbo.
//!
//! kurbo fits best to a smooth source, so the run is first turned into a
//! Catmull-Rom spline through its points, and kurbo then fits cubics to that
//! spline. (Fitting the polyline directly gives about 4× the curves;
//! kurbo's optimising fit gives fewer, but is several hundred times slower.)

use flatart_domain::{CubicBezier, PxPoint};
use kurbo::simplify::SimplifyBezPath;
use kurbo::{BezPath, Point, fit_to_bezpath};

fn point(p: PxPoint) -> Point {
    Point::new(p.x.0, p.y.0)
}

fn px(p: Point) -> PxPoint {
    PxPoint::new(p.x, p.y)
}

/// A centripetal Catmull-Rom spline through `points` (at least 2). Unlike
/// the uniform kind it doesn't overshoot where the spacing changes, so it
/// stays close to the polyline. A `smooth_loop` (first point equals last)
/// wraps round, so it has no kink where it starts.
fn spline(points: &[PxPoint], smooth_loop: bool) -> BezPath {
    let n = points.len() as isize;
    // A neighbour index, or None past an open run's ends.
    let at = |i: isize| -> Option<Point> {
        let i = if smooth_loop {
            i.rem_euclid(n - 1)
        } else if (0..n).contains(&i) {
            i
        } else {
            return None;
        };
        Some(point(points[i as usize]))
    };
    let mut path = BezPath::new();
    path.move_to(point(points[0]));
    for i in 0..n - 1 {
        let (Some(p1), Some(p2)) = (at(i), at(i + 1)) else {
            continue;
        };
        let chord = p2 - p1;
        let d2 = chord.hypot().sqrt();
        // The tangent at each end, scaled to this segment; along the chord
        // where there's no neighbour.
        let start = at(i - 1).map_or(chord, |p0| {
            let d1 = (p1 - p0).hypot().sqrt();
            ((p1 - p0) / d1 - (p2 - p0) / (d1 + d2)) * d2 + chord
        });
        let end = at(i + 2).map_or(chord, |p3| {
            let d3 = (p3 - p2).hypot().sqrt();
            ((p3 - p2) / d3 - (p3 - p1) / (d2 + d3)) * d2 + chord
        });
        path.curve_to(p1 + start / 3.0, p2 - end / 3.0, p2);
    }
    path
}

/// Fits cubics to `points` (at least 2) aiming for `accuracy` from the
/// spline through them. The ends are exactly the run's ends and the curves
/// join exactly. `smooth_loop` is for a closed run with no corner where it
/// starts.
pub fn fit(points: &[PxPoint], accuracy: f64, smooth_loop: bool) -> Vec<CubicBezier> {
    let source = SimplifyBezPath::new(spline(points, smooth_loop).iter());
    let fitted = fit_to_bezpath(&source, accuracy);
    let mut curves: Vec<CubicBezier> = fitted
        .segments()
        .map(|seg| {
            let c = seg.to_cubic();
            CubicBezier {
                from: px(c.p0),
                ctrl1: px(c.p1),
                ctrl2: px(c.p2),
                to: px(c.p3),
            }
        })
        .collect();
    // kurbo evaluates the ends, so pin them to the exact input.
    if let (Some(first), Some(&start)) = (curves.first_mut(), points.first()) {
        first.from = start;
    }
    if let (Some(last), Some(&end)) = (curves.last_mut(), points.last()) {
        last.to = end;
    }
    for i in 1..curves.len() {
        curves[i].from = curves[i - 1].to;
    }
    curves
}
