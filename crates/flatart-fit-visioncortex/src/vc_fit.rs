//! One corner-free run, fitted by visioncortex.
//!
//! visioncortex fits a Schneider-style least-squares cubic chain (from
//! `flo_curves`) straight to the polyline, after adding points along long
//! segments so a curve can't bulge between two far-apart points unseen.
//!
//! vtracer also cuts each run at its inflections before fitting. That suits
//! its smoothed paths, but the tracer's pixel-snapped borders bend both ways
//! at almost every point, so cutting there gave more curves than kurbo, and
//! it isn't done here.

use flatart_domain::{CubicBezier, PxPoint};
use visioncortex::{PointF64, SubdivideSmooth};

fn point(p: PxPoint) -> PointF64 {
    PointF64::new(p.x.0, p.y.0)
}

fn px(p: PointF64) -> PxPoint {
    PxPoint::new(p.x, p.y)
}

/// Fits cubics to `points` (at least 3) aiming for `accuracy`, as two
/// candidates: with its joins smoothed (see [`smooth_joins`]), then as
/// visioncortex fitted it, for when smoothing pushes it out of tolerance.
/// The ends are exactly the run's ends and the curves join exactly.
///
/// visioncortex fits open runs only, so a closed run is fitted as two
/// halves.
pub fn fit(points: &[PxPoint], accuracy: f64, smooth_loop: bool) -> Vec<Vec<CubicBezier>> {
    let n = points.len();
    let raw = if points[0] == points[n - 1] {
        let mid = n / 2;
        let mut curves = fit_open(&points[..=mid], accuracy);
        curves.extend(fit_open(&points[mid..], accuracy));
        curves
    } else {
        fit_open(points, accuracy)
    };
    let mut smoothed = raw.clone();
    smooth_joins(&mut smoothed, smooth_loop);
    vec![smoothed, raw]
}

/// Turns both handles at each join onto their average direction, keeping
/// their lengths, so the run has no kink. A run has no corner, so every
/// join should be smooth, but visioncortex fits long runs in blocks of 100
/// points and welds the blocks where they meet, with a kink, and a closed
/// run is fitted in halves. `smooth_loop` also smooths where a closed run
/// meets itself.
fn smooth_joins(curves: &mut [CubicBezier], smooth_loop: bool) {
    let n = curves.len();
    let joins = if smooth_loop { n } else { n.saturating_sub(1) };
    for j in 0..joins {
        let (i, k) = (j, (j + 1) % n);
        let (a, b) = (curves[i], curves[k]);
        let u = (a.to.x.0 - a.ctrl2.x.0, a.to.y.0 - a.ctrl2.y.0);
        let v = (b.ctrl1.x.0 - b.from.x.0, b.ctrl1.y.0 - b.from.y.0);
        let (lu, lv) = (u.0.hypot(u.1), v.0.hypot(v.1));
        if lu == 0.0 || lv == 0.0 {
            continue;
        }
        let d = (u.0 / lu + v.0 / lv, u.1 / lu + v.1 / lv);
        let ld = d.0.hypot(d.1);
        if ld < 1e-9 {
            continue;
        }
        let d = (d.0 / ld, d.1 / ld);
        curves[i].ctrl2 = PxPoint::new(a.to.x.0 - d.0 * lu, a.to.y.0 - d.1 * lu);
        curves[k].ctrl1 = PxPoint::new(b.from.x.0 + d.0 * lv, b.from.y.0 + d.1 * lv);
    }
}

/// Fits an open run, its ends pinned exactly (visioncortex pins them too;
/// this makes it exact whatever the conversion does).
fn fit_open(points: &[PxPoint], accuracy: f64) -> Vec<CubicBezier> {
    let (start, end) = (points[0], points[points.len() - 1]);
    let path: Vec<PointF64> = points.iter().copied().map(point).collect();
    let mut curves: Vec<CubicBezier> = SubdivideSmooth::fit_points_with_beziers(&path, accuracy)
        .into_iter()
        .map(|[from, ctrl1, ctrl2, to]| CubicBezier {
            from: px(from),
            ctrl1: px(ctrl1),
            ctrl2: px(ctrl2),
            to: px(to),
        })
        .collect();
    if let Some(first) = curves.first_mut() {
        first.from = start;
    }
    if let Some(last) = curves.last_mut() {
        last.to = end;
    }
    curves
}
