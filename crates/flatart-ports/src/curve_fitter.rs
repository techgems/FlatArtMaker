use flatart_domain::settings::FitSettings;
use flatart_domain::{CubicBezier, PxPoint, Result};

/// Fits cubic Béziers to a polyline (Pipeline step 7).
///
/// # Contract
/// - **Input:** at least 2 points, in image space. Consecutive points differ.
/// - **Ends fixed:** the first curve starts exactly at the first point, and the
///   last curve ends exactly at the last point.
/// - **Joined:** each curve starts exactly where the previous one ends.
/// - **Tolerance:** every input point lies within `settings.tolerance` of the
///   curves, and every point of the curves lies within it of the polyline.
/// - **Corners:** where the polyline turns by more than `settings.corner`, the
///   turning point is exactly the end of one curve and the start of the next.
/// - **Straight stays straight:** collinear input gives curves whose control
///   points lie on that line.
/// - **Deterministic.**
pub trait CurveFitter {
    fn fit(&self, points: &[PxPoint], settings: &FitSettings) -> Result<Vec<CubicBezier>>;
}
