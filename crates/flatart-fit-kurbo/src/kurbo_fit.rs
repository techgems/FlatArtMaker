//! One corner-free run, fitted by kurbo.
//!
//! kurbo fits best to a smooth source, so the run is first turned into a
//! Catmull-Rom spline through its points, and kurbo then fits cubics to that
//! spline. (Fitting the polyline directly gives about 4× the curves;
//! kurbo's optimising fit gives fewer, but is several hundred times slower.)

use flatart_domain::{CubicBezier, PxPoint};
use kurbo::simplify::SimplifyBezPath;
use kurbo::{BezPath, PathSeg, Point, fit_to_bezpath};

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

/// Whether the segment is a straight line: its control points lie on the
/// line between its ends.
fn is_straight(seg: &PathSeg) -> bool {
    let c = seg.to_cubic();
    let d = c.p3 - c.p0;
    let len2 = d.hypot2();
    let off = |p: Point| (p - c.p0).cross(d).abs();
    len2 > 0.0 && off(c.p1) <= 1e-12 * len2 && off(c.p2) <= 1e-12 * len2
}

/// Fits cubics to `points` (at least 2) aiming for `accuracy` from the
/// spline through them. The ends are exactly the run's ends and the curves
/// join exactly. `smooth_loop` is for a closed run with no corner where it
/// starts.
///
/// Straight stretches of the spline become exact lines, and kurbo fits only
/// the curved stretches between them: kurbo's fit never finishes on an
/// exactly straight source, which pixel staircases give often.
pub fn fit(points: &[PxPoint], accuracy: f64, smooth_loop: bool) -> Vec<CubicBezier> {
    let segments: Vec<PathSeg> = spline(points, smooth_loop).segments().collect();
    let mut curves = Vec::new();
    let mut i = 0;
    while i < segments.len() {
        let straight = is_straight(&segments[i]);
        let mut j = i + 1;
        while j < segments.len() && is_straight(&segments[j]) == straight {
            j += 1;
        }
        // Segment k runs from points[k] to points[k + 1].
        if straight {
            curves.extend(lines(&points[i..=j]));
        } else {
            curves.extend(fit_curved(&segments[i..j], &points[i..=j], accuracy));
        }
        i = j;
    }
    for i in 1..curves.len() {
        curves[i].from = curves[i - 1].to;
    }
    curves
}

/// Straight lines through `points`, one per run of points that carry on
/// in the same direction.
fn lines(points: &[PxPoint]) -> Vec<CubicBezier> {
    let dir = |a: PxPoint, b: PxPoint| (b.x.0 - a.x.0, b.y.0 - a.y.0);
    let mut lines = Vec::new();
    let mut start = 0;
    for k in 1..points.len() - 1 {
        let (u, v) = (dir(points[start], points[k]), dir(points[k], points[k + 1]));
        let cross = u.0 * v.1 - u.1 * v.0;
        let dot = u.0 * v.0 + u.1 * v.1;
        let scale = (u.0 * u.0 + u.1 * u.1) * (v.0 * v.0 + v.1 * v.1);
        if cross * cross > 1e-24 * scale || dot <= 0.0 {
            lines.push(CubicBezier::line(points[start], points[k]));
            start = k;
        }
    }
    lines.push(CubicBezier::line(points[start], points[points.len() - 1]));
    lines
}

/// Fits a curved stretch (`points` are its segments' ends). kurbo also never
/// finishes on a source whose ends meet (no chord), so a closed stretch is
/// fitted as two halves.
fn fit_curved(segments: &[PathSeg], points: &[PxPoint], accuracy: f64) -> Vec<CubicBezier> {
    let n = segments.len();
    if n > 1 && points[0] == points[n] {
        let mid = n / 2;
        let mut curves = fit_part(&segments[..mid], accuracy, points[0], points[mid]);
        curves.extend(fit_part(&segments[mid..], accuracy, points[mid], points[n]));
        return curves;
    }
    fit_part(segments, accuracy, points[0], points[n])
}

/// Fits one stretch of the spline, its ends pinned to `start` and `end`.
fn fit_part(segments: &[PathSeg], accuracy: f64, start: PxPoint, end: PxPoint) -> Vec<CubicBezier> {
    let source = SimplifyBezPath::new(BezPath::from_path_segments(segments.iter().copied()).iter());
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
    if let Some(first) = curves.first_mut() {
        first.from = start;
    }
    if let Some(last) = curves.last_mut() {
        last.to = end;
    }
    curves
}
