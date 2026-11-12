//! The borders' Béziers in model space: mm, y up, placed like the 3MF (the
//! outline's lowest x and y at 0).

use flatart_domain::{BorderEnds, BorderMap, CubicBezier, Label};

/// A point in model space (mm).
pub type P2 = (f64, f64);

/// A cubic Bézier in model space: start, two controls, end.
pub type Cubic = [P2; 4];

/// The borders in model space, plus where each one starts and ends.
pub struct Placed {
    /// Each border's curves, joined end to end. Never empty, and no curve
    /// starts where it ends.
    pub curves: Vec<Vec<Cubic>>,
    /// Each border's start and end point ids: its junctions, or, for a
    /// closed border, one id of its own for both.
    pub ends: Vec<(u32, u32)>,
}

fn eval(c: &Cubic, t: f64) -> P2 {
    let s = 1.0 - t;
    let (a, b, d, e) = (s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t);
    (
        a * c[0].0 + b * c[1].0 + d * c[2].0 + e * c[3].0,
        a * c[0].1 + b * c[1].1 + d * c[2].1 + e * c[3].1,
    )
}

/// The lowest and highest value of one coordinate along the curve.
fn range(c: &Cubic, coord: impl Fn(P2) -> f64) -> (f64, f64) {
    let [p0, p1, p2, p3] = c.map(&coord);
    // The derivative's roots: a t² + b t + c = 0, divided by 3.
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 2.0 * (p0 - 2.0 * p1 + p2);
    let k = p1 - p0;
    let mut ts = vec![0.0, 1.0];
    if a.abs() > 1e-12 {
        let disc = b * b - 4.0 * a * k;
        if disc >= 0.0 {
            let r = disc.sqrt();
            ts.extend([(-b + r) / (2.0 * a), (-b - r) / (2.0 * a)]);
        }
    } else if b.abs() > 1e-12 {
        ts.push(-k / b);
    }
    ts.into_iter()
        .filter(|t| (0.0..=1.0).contains(t))
        .map(|t| coord(eval(c, t)))
        .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)))
}

/// Splits a curve in half (de Casteljau).
fn halves(c: &Cubic) -> [Cubic; 2] {
    let mid = |a: P2, b: P2| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let (ab, bc, cd) = (mid(c[0], c[1]), mid(c[1], c[2]), mid(c[2], c[3]));
    let (abc, bcd) = (mid(ab, bc), mid(bc, cd));
    let m = mid(abc, bcd);
    [[c[0], ab, abc, m], [m, bcd, cd, c[3]]]
}

/// Places every border at `scale` mm per pixel.
pub fn place(map: &BorderMap, scale: f64) -> Placed {
    let px =
        |c: &CubicBezier| -> Cubic { [c.from, c.ctrl1, c.ctrl2, c.to].map(|p| (p.x.0, p.y.0)) };
    // Place by the outline: the borders that touch outside.
    let (mut min_x, mut max_y) = (f64::MAX, f64::MIN);
    for border in &map.borders {
        if border.left != Label::OUTSIDE && border.right != Label::OUTSIDE {
            continue;
        }
        for c in &border.curve {
            let c = px(c);
            min_x = min_x.min(range(&c, |p| p.0).0);
            max_y = max_y.max(range(&c, |p| p.1).1);
        }
    }
    if min_x == f64::MAX {
        (min_x, max_y) = (0.0, map.height.0);
    }
    let to_model = |p: P2| ((p.0 - min_x) * scale, (max_y - p.1) * scale);
    let closed_base = map.junctions.len() as u32;
    let mut curves = Vec::with_capacity(map.borders.len());
    let mut ends = Vec::with_capacity(map.borders.len());
    for (b, border) in map.borders.iter().enumerate() {
        let mut line = Vec::with_capacity(border.curve.len());
        for c in &border.curve {
            let c = px(c).map(to_model);
            // An edge can't start where it ends, so a curve that closes on
            // itself becomes two.
            if c[0] == c[3] {
                line.extend(halves(&c));
            } else {
                line.push(c);
            }
        }
        curves.push(line);
        ends.push(match border.ends {
            BorderEnds::Open { start, end } => (start.0, end.0),
            BorderEnds::Closed => (closed_base + b as u32, closed_base + b as u32),
        });
    }
    Placed { curves, ends }
}
