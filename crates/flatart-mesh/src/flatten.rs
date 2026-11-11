//! Each border into one polyline in mm, once.
//!
//! Junctions get one point each, shared by every border that ends there, so
//! everything built from these points meets exactly.

use flatart_domain::{BorderEnds, BorderMap, CubicBezier, Label};

/// The most segments one curve is split into.
const MAX_SEGMENTS: usize = 1000;

/// Every point, in model space (mm, y up), and each border as point ids.
pub struct Outline {
    pub points: Vec<(f64, f64)>,
    /// Each border's points, start to end. A closed border ends at its
    /// first point's id.
    pub borders: Vec<Vec<u32>>,
    /// Whether each point is a junction.
    pub junction: Vec<bool>,
}

/// How many straight segments keep `curve` within `tolerance` (scaled by
/// `scale`): the chord error of n even steps is at most |B''| / 8n², and
/// |B''| ≤ 6 × the largest second difference of the control points.
fn segments(curve: &CubicBezier, scale: f64, tolerance: f64) -> usize {
    let p = [curve.from, curve.ctrl1, curve.ctrl2, curve.to].map(|q| (q.x.0, q.y.0));
    let second = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
        (a.0 - 2.0 * b.0 + c.0).hypot(a.1 - 2.0 * b.1 + c.1)
    };
    let m = second(p[0], p[1], p[2]).max(second(p[1], p[2], p[3])) * scale;
    let n = (0.75 * m / tolerance).sqrt().ceil();
    if n.is_finite() && n >= 1.0 {
        (n as usize).min(MAX_SEGMENTS)
    } else {
        1
    }
}

/// The points along a border's curves in image pixels, ends included.
fn polyline(curves: &[CubicBezier], scale: f64, tolerance: f64) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    for (i, curve) in curves.iter().enumerate() {
        let n = segments(curve, scale, tolerance);
        let first = usize::from(i > 0);
        points.extend((first..=n).map(|k| {
            let p = curve.point_at(k as f64 / n as f64);
            (p.x.0, p.y.0)
        }));
    }
    points.dedup();
    points
}

/// Flattens every border at `tolerance` mm, scaled by `scale` mm per pixel,
/// and moves the outline's lowest x and y to 0.
pub fn flatten(map: &BorderMap, scale: f64, tolerance: f64) -> Outline {
    let mut px: Vec<(f64, f64)> = map.junctions.iter().map(|j| (j.at.x.0, j.at.y.0)).collect();
    let mut junction = vec![true; px.len()];
    let mut borders = Vec::with_capacity(map.borders.len());
    for border in &map.borders {
        let line = polyline(&border.curve, scale, tolerance);
        let inner = &line[1..line.len().saturating_sub(1)];
        let mut ids = Vec::with_capacity(line.len());
        let (start, end) = match border.ends {
            BorderEnds::Open { start, end } => (start.0, end.0),
            BorderEnds::Closed => {
                let id = px.len() as u32;
                px.push(line[0]);
                junction.push(false);
                (id, id)
            }
        };
        ids.push(start);
        for &p in inner {
            ids.push(px.len() as u32);
            px.push(p);
            junction.push(false);
        }
        ids.push(end);
        borders.push(ids);
    }

    // Place by the outline: the borders that touch outside.
    let outline = map
        .borders
        .iter()
        .zip(&borders)
        .filter(|(b, _)| b.left == Label::OUTSIDE || b.right == Label::OUTSIDE)
        .flat_map(|(_, ids)| ids.iter().map(|&i| px[i as usize]));
    let (mut min_x, mut max_y) = (f64::MAX, f64::MIN);
    for (x, y) in outline {
        min_x = min_x.min(x);
        max_y = max_y.max(y);
    }
    if min_x == f64::MAX {
        (min_x, max_y) = (0.0, map.height.0);
    }
    let points = px
        .into_iter()
        .map(|(x, y)| ((x - min_x) * scale, (max_y - y) * scale))
        .collect();
    Outline {
        points,
        borders,
        junction,
    }
}
