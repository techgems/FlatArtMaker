//! Where the art sits on the page, in mm.

use flatart_domain::{BorderMap, PxPoint};

/// Maps image pixels to page millimetres: the outline's top-left corner at
/// the origin, `scale` mm per pixel. SVG's y runs down, like the image's.
pub struct Placement {
    origin: (f64, f64),
    scale: f64,
    /// The page size in mm.
    pub width: f64,
    pub height: f64,
}

impl Placement {
    /// Fits the page to the outline: the curve ends of the borders that touch
    /// outside (every border when none does).
    pub fn new(map: &BorderMap, scale: f64) -> Self {
        let touches_outside =
            |b: &&flatart_domain::Border| b.left.is_outside() || b.right.is_outside();
        let outline: Vec<_> = if map.borders.iter().any(|b| touches_outside(&b)) {
            map.borders.iter().filter(touches_outside).collect()
        } else {
            map.borders.iter().collect()
        };
        let (mut lo, mut hi) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
        for c in outline.iter().flat_map(|b| &b.curve) {
            for p in [c.from, c.to] {
                lo = (lo.0.min(p.x.0), lo.1.min(p.y.0));
                hi = (hi.0.max(p.x.0), hi.1.max(p.y.0));
            }
        }
        if lo.0 > hi.0 {
            (lo, hi) = ((0.0, 0.0), (map.width.0, map.height.0));
        }
        Self {
            origin: lo,
            scale,
            width: (hi.0 - lo.0) * scale,
            height: (hi.1 - lo.1) * scale,
        }
    }

    pub fn place(&self, p: PxPoint) -> (f64, f64) {
        (
            (p.x.0 - self.origin.0) * self.scale,
            (p.y.0 - self.origin.1) * self.scale,
        )
    }
}

/// A number with up to 4 decimals and no trailing zeros, for compact,
/// stable output.
pub fn number(v: f64) -> String {
    let s = format!("{:.4}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    match s {
        "-0" | "" => "0".into(),
        s => s.into(),
    }
}
