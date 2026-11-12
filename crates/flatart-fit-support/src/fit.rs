//! Ties corner splitting, the library's fit and the checks together.

use crate::{corners, deviation, hooks};
use flatart_domain::settings::FitSettings;
use flatart_domain::{CubicBezier, Error, PxPoint, Result};

/// The target accuracy passed to the library, as fractions of the
/// tolerance, tried in turn. Libraries measure their error approximately, so
/// they aim inside the tolerance.
const ACCURACY_STEPS: [f64; 2] = [0.8, 0.4];

/// Fits cubics to `points` under the `CurveFitter` contract.
///
/// `fit_run(run, accuracy, smooth_loop)` is the library's part. It fits one
/// corner-free run (at least 3 points, not straight) aiming for `accuracy`,
/// and returns one or more candidate fits, best first. Each candidate's
/// curves must start exactly at the run's first point, end exactly at its
/// last, and join exactly. `smooth_loop` means the run is a whole closed
/// loop (first point equals last) with no corner where it starts.
///
/// Each candidate is checked against the tolerance and for doubling back,
/// and the first that passes is used. When none does, the run is fitted
/// again at the next accuracy step, and when all fail it's split in half.
pub fn fit(
    points: &[PxPoint],
    settings: &FitSettings,
    fit_run: impl Fn(&[PxPoint], f64, bool) -> Vec<Vec<CubicBezier>>,
) -> Result<Vec<CubicBezier>> {
    validate(points)?;
    let tolerance = settings.tolerance.get().0;
    let corner = settings.corner.get().0;
    let runs = corners::runs(points, corner);
    // A loop with no corner at all, not even where it starts, is fitted
    // round its start smoothly.
    let smooth_loop = runs.len() == 1 && corners::smooth_seam(points, corner);
    let mut curves = Vec::new();
    for run in runs {
        fit_checked(&fit_run, run, tolerance, smooth_loop, &mut curves);
    }
    Ok(curves)
}

fn validate(points: &[PxPoint]) -> Result<()> {
    if points.len() < 2 {
        return Err(Error::InvalidInput(format!(
            "curve fitting needs at least 2 points, got {}",
            points.len()
        )));
    }
    if points
        .iter()
        .any(|p| !p.x.0.is_finite() || !p.y.0.is_finite())
    {
        return Err(Error::InvalidInput(
            "curve fitting got a point that isn't finite".into(),
        ));
    }
    if let Some(i) = points.windows(2).position(|w| w[0] == w[1]) {
        return Err(Error::InvalidInput(format!(
            "curve fitting got the same point twice in a row at {i}"
        )));
    }
    Ok(())
}

/// Fits one corner-free run onto `out`: one line if it's straight, else the
/// library's first candidate that passes the checks, else its two halves.
fn fit_checked<F: Fn(&[PxPoint], f64, bool) -> Vec<Vec<CubicBezier>>>(
    fit_run: &F,
    run: &[PxPoint],
    tolerance: f64,
    smooth_loop: bool,
    out: &mut Vec<CubicBezier>,
) {
    let (first, last) = (run[0], run[run.len() - 1]);
    if run.len() == 2 || straight(run) {
        out.push(CubicBezier::line(first, last));
        return;
    }
    for step in ACCURACY_STEPS {
        for curves in fit_run(run, tolerance * step, smooth_loop) {
            if deviation::within(&curves, run, tolerance) && hooks::none(&curves, run) {
                out.extend(curves);
                return;
            }
        }
    }
    let mid = run.len() / 2;
    fit_checked(fit_run, &run[..=mid], tolerance, false, out);
    fit_checked(fit_run, &run[mid..], tolerance, false, out);
}

/// Whether every point lies on the segment from the first to the last, in
/// order, so one line covers the run exactly.
fn straight(run: &[PxPoint]) -> bool {
    let (a, b) = (run[0], run[run.len() - 1]);
    let (dx, dy) = (b.x.0 - a.x.0, b.y.0 - a.y.0);
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        return false;
    }
    let mut along = 0.0;
    run.iter().skip(1).all(|p| {
        let (px, py) = (p.x.0 - a.x.0, p.y.0 - a.y.0);
        let t = (px * dx + py * dy) / len2;
        let off = (px * dy - py * dx).abs() / len2.sqrt();
        let ahead = off <= 1e-9 && t > along;
        along = t;
        ahead
    })
}
