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
/// - **No doubling back:** no curve runs back behind its start or past its
///   end, measured along the line between them. The first curve sets off,
///   and the last arrives, the way the polyline runs at that end, and where
///   two curves meet the second carries on rather than turning back over
///   the first, except at a corner or where the polyline itself turns back
///   (more than 90°). (A curve that turns back within the tolerance is a
///   needle a hair wide; flattened, its two sides can cross.) A curve's
///   direction at an end is along its handle there, or past it to the next
///   control point when the handle has no length.
/// - **Deterministic.**
pub trait CurveFitter {
    fn fit(&self, points: &[PxPoint], settings: &FitSettings) -> Result<Vec<CubicBezier>>;
}
