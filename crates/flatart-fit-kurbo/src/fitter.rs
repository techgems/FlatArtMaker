//! The adapter: the shared fit loop, with kurbo fitting each run.

use crate::kurbo_fit;
use flatart_domain::settings::FitSettings;
use flatart_domain::{CubicBezier, PxPoint, Result};
use flatart_ports::CurveFitter;

/// Fits with kurbo, splitting at corners first.
#[derive(Debug, Clone, Copy, Default)]
pub struct KurboFitter;

impl CurveFitter for KurboFitter {
    fn fit(&self, points: &[PxPoint], settings: &FitSettings) -> Result<Vec<CubicBezier>> {
        flatart_fit_support::fit(points, settings, |run, accuracy, smooth_loop| {
            vec![kurbo_fit::fit(run, accuracy, smooth_loop)]
        })
    }
}
