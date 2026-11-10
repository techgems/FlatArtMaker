//! Splitting a polyline at its corners.

use flatart_domain::PxPoint;

/// How far the polyline turns at `b`, in degrees (0 = straight on,
/// 180 = straight back).
fn turn(a: PxPoint, b: PxPoint, c: PxPoint) -> f64 {
    let (ux, uy) = (b.x.0 - a.x.0, b.y.0 - a.y.0);
    let (vx, vy) = (c.x.0 - b.x.0, c.y.0 - b.y.0);
    let cross = ux * vy - uy * vx;
    let dot = ux * vx + uy * vy;
    cross.abs().atan2(dot).to_degrees()
}

/// Whether `points` is a closed loop (first equals last) that doesn't turn
/// by more than `corner_deg` where it starts.
pub fn smooth_seam(points: &[PxPoint], corner_deg: f64) -> bool {
    let n = points.len();
    n > 3 && points[0] == points[n - 1] && turn(points[n - 2], points[0], points[1]) <= corner_deg
}

/// The runs between corners (turns sharper than `corner_deg`). Each run ends
/// at the point the next one starts with.
pub fn runs(points: &[PxPoint], corner_deg: f64) -> Vec<&[PxPoint]> {
    let mut runs = Vec::new();
    let mut start = 0;
    for i in 1..points.len().saturating_sub(1) {
        if turn(points[i - 1], points[i], points[i + 1]) > corner_deg {
            runs.push(&points[start..=i]);
            start = i;
        }
    }
    runs.push(&points[start..]);
    runs
}
