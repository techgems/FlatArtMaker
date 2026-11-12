//! The adapter: the shared fit loop, with visioncortex fitting each run.

use crate::vc_fit;
use flatart_domain::settings::FitSettings;
use flatart_domain::{CubicBezier, PxPoint, Result};
use flatart_ports::CurveFitter;

/// Fits with visioncortex, splitting at corners first.
#[derive(Debug, Clone, Copy, Default)]
pub struct VisionCortexFitter;

impl CurveFitter for VisionCortexFitter {
    fn fit(&self, points: &[PxPoint], settings: &FitSettings) -> Result<Vec<CubicBezier>> {
        flatart_fit_support::fit(points, settings, vc_fit::fit)
    }
}
