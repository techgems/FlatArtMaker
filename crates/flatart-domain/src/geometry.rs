//! Points, curves and polygons.
//!
//! Two coordinate spaces:
//! - **image space** (`PxPoint`): source-image pixels, origin top-left, y down
//! - **model space** (`MmPoint`, `MmPoint3`): millimetres, x right, y up, z up

use crate::{Mm, Px};
use serde::{Deserialize, Serialize};

/// A point in image space.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct PxPoint {
    pub x: Px,
    pub y: Px,
}

impl PxPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x: Px(x), y: Px(y) }
    }

    fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            self.x.0 + (other.x.0 - self.x.0) * t,
            self.y.0 + (other.y.0 - self.y.0) * t,
        )
    }
}

/// A cubic Bézier in image space.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct CubicBezier {
    pub from: PxPoint,
    pub ctrl1: PxPoint,
    pub ctrl2: PxPoint,
    pub to: PxPoint,
}

impl CubicBezier {
    /// A straight segment as a cubic.
    pub fn line(from: PxPoint, to: PxPoint) -> Self {
        Self {
            from,
            ctrl1: from.lerp(to, 1.0 / 3.0),
            ctrl2: from.lerp(to, 2.0 / 3.0),
            to,
        }
    }

    /// The point at `t` (0 to 1), by de Casteljau.
    pub fn point_at(&self, t: f64) -> PxPoint {
        let a = self.from.lerp(self.ctrl1, t);
        let b = self.ctrl1.lerp(self.ctrl2, t);
        let c = self.ctrl2.lerp(self.to, t);
        let d = a.lerp(b, t);
        let e = b.lerp(c, t);
        d.lerp(e, t)
    }

    /// The same curve, run the other way.
    pub fn reversed(&self) -> Self {
        Self {
            from: self.to,
            ctrl1: self.ctrl2,
            ctrl2: self.ctrl1,
            to: self.from,
        }
    }
}

/// A point in model space on the XY plane.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct MmPoint {
    pub x: Mm,
    pub y: Mm,
}

impl MmPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x: Mm(x), y: Mm(y) }
    }
}

/// A point in model space.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct MmPoint3 {
    pub x: Mm,
    pub y: Mm,
    pub z: Mm,
}

impl MmPoint3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Mm(x),
            y: Mm(y),
            z: Mm(z),
        }
    }
}

/// A flat polygon with holes, in model space. Rings are implicitly closed (the
/// last point is not repeated) and may run either way round.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PolygonWithHoles {
    pub outer: Vec<MmPoint>,
    pub holes: Vec<Vec<MmPoint>>,
}

impl PolygonWithHoles {
    /// Every vertex: the outer ring, then each hole in order. Triangle indices
    /// from a `Triangulator` point into this sequence.
    pub fn vertices(&self) -> impl Iterator<Item = MmPoint> + '_ {
        self.outer
            .iter()
            .chain(self.holes.iter().flatten())
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_passes_through_its_midpoint() {
        let line = CubicBezier::line(PxPoint::new(0.0, 0.0), PxPoint::new(3.0, 6.0));
        let mid = line.point_at(0.5);
        assert!((mid.x.0 - 1.5).abs() < 1e-12 && (mid.y.0 - 3.0).abs() < 1e-12);
    }

    #[test]
    fn reversed_swaps_the_ends() {
        let line = CubicBezier::line(PxPoint::new(0.0, 0.0), PxPoint::new(1.0, 0.0));
        let back = line.reversed();
        assert_eq!(back.from, line.to);
        let (a, b) = (back.point_at(0.25), line.point_at(0.75));
        assert!((a.x.0 - b.x.0).abs() < 1e-12 && (a.y.0 - b.y.0).abs() < 1e-12);
    }

    #[test]
    fn vertices_list_outer_then_holes() {
        let poly = PolygonWithHoles {
            outer: vec![MmPoint::new(0.0, 0.0), MmPoint::new(1.0, 0.0)],
            holes: vec![vec![MmPoint::new(0.5, 0.5)]],
        };
        let all: Vec<_> = poly.vertices().collect();
        assert_eq!(all.len(), 3);
        assert_eq!(all[2], MmPoint::new(0.5, 0.5));
    }
}
