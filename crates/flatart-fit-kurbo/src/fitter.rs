//! Ties corner splitting, kurbo fitting and the tolerance check together.

use crate::{corners, deviation, hooks, kurbo_fit};
use flatart_domain::settings::FitSettings;
use flatart_domain::{CubicBezier, Error, PxPoint, Result};
use flatart_ports::CurveFitter;

/// kurbo's target accuracy, as fractions of the tolerance, tried in turn.
/// kurbo measures its error approximately, so it aims inside the tolerance.
const ACCURACY_STEPS: [f64; 2] = [0.8, 0.4];

/// Fits with kurbo, splitting at corners first.
#[derive(Debug, Clone, Copy, Default)]
pub struct KurboFitter;

impl CurveFitter for KurboFitter {
    fn fit(&self, points: &[PxPoint], settings: &FitSettings) -> Result<Vec<CubicBezier>> {
        validate(points)?;
        let tolerance = settings.tolerance.get().0;
        let corner = settings.corner.get().0;
        let runs = corners::runs(points, corner);
        // A loop with no corner at all, not even where it starts, is fitted
        // round its start smoothly.
        let smooth_loop = runs.len() == 1 && corners::smooth_seam(points, corner);
        let mut curves = Vec::new();
        for run in runs {
            fit_run(run, tolerance, smooth_loop, &mut curves);
        }
        Ok(curves)
    }
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

/// Fits one corner-free run onto `out`: one line if it's straight, else
/// kurbo, else (if kurbo can't stay within `tolerance`) its two halves.
/// `smooth_loop` is for a whole closed run with no corner where it starts.
fn fit_run(run: &[PxPoint], tolerance: f64, smooth_loop: bool, out: &mut Vec<CubicBezier>) {
    let (first, last) = (run[0], run[run.len() - 1]);
    if run.len() == 2 || straight(run) {
        out.push(CubicBezier::line(first, last));
        return;
    }
    for step in ACCURACY_STEPS {
        let curves = kurbo_fit::fit(run, tolerance * step, smooth_loop);
        if deviation::within(&curves, run, tolerance) && hooks::none(&curves, run) {
            out.extend(curves);
            return;
        }
    }
    let mid = run.len() / 2;
    fit_run(&run[..=mid], tolerance, false, out);
    fit_run(&run[mid..], tolerance, false, out);
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
