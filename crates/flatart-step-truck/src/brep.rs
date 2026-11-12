//! Closed solids from a slab, with truck: one straight extrusion per area.
//!
//! An area's loops give its flat bottom and top. Each curve along them
//! gives one vertical wall, and walls meet at vertical lines through the
//! curves' ends. Every edge is one exact Bézier (or line) or one of those
//! vertical lines. Each loop gets its own vertices, so areas that touch at
//! a point, or an area that touches itself, still give closed manifold
//! solids.

use crate::loops::{Area, Step, areas, oriented};
use crate::parts::Slab;
use crate::place::{Cubic, P2, Placed};
use flatart_domain::{Error, Result, Stage};
use truck_modeling::{
    BSplineCurve, BSplineSurface, Curve, Edge, Face, KnotVec, Line, Plane, Point3, Shell, Solid,
    Surface, Vertex, Wire,
};

fn failed(message: String) -> Error {
    Error::StageFailed {
        stage: Stage::Export,
        message,
    }
}

/// One solid, with what orders it among its component's bodies.
pub struct Body {
    pub solid: Solid,
    /// The footprint's area (mm²).
    pub area: f64,
    /// The footprint's highest y and lowest x.
    pub top: f64,
    pub left: f64,
}

fn p3(p: P2, z: f64) -> Point3 {
    Point3::new(p.0, p.1, z)
}

/// Whether the curve is a straight line from its start to its end.
fn straight(c: &Cubic) -> bool {
    let d = (c[3].0 - c[0].0, c[3].1 - c[0].1);
    let len2 = d.0 * d.0 + d.1 * d.1;
    let off = |p: P2| ((p.0 - c[0].0) * d.1 - (p.1 - c[0].1) * d.0).abs();
    len2 > 0.0 && off(c[1]) <= 1e-12 * len2 && off(c[2]) <= 1e-12 * len2
}

fn curve(c: &Cubic, z: f64) -> Curve {
    if straight(c) {
        Curve::Line(Line(p3(c[0], z), p3(c[3], z)))
    } else {
        Curve::BSplineCurve(BSplineCurve::new(
            KnotVec::bezier_knot(3),
            c.iter().map(|&p| p3(p, z)).collect(),
        ))
    }
}

/// The vertical wall under `c` from `lo` to `hi`, facing the curve's right
/// (the material is on its left).
fn wall(c: &Cubic, lo: f64, hi: f64) -> Surface {
    if straight(c) {
        Surface::Plane(Plane::new(p3(c[0], lo), p3(c[3], lo), p3(c[0], hi)))
    } else {
        Surface::BSplineSurface(BSplineSurface::new(
            (KnotVec::bezier_knot(3), KnotVec::bezier_knot(1)),
            c.iter().map(|&p| vec![p3(p, lo), p3(p, hi)]).collect(),
        ))
    }
}

fn edge(front: &Vertex, back: &Vertex, curve: Curve) -> Result<Edge> {
    Edge::try_new(front, back, curve).map_err(|e| failed(format!("a STEP edge is invalid: {e:?}")))
}

/// One loop's edges and walls.
struct Ring {
    bottom: Wire,
    top: Wire,
    walls: Vec<Face>,
}

/// Builds one loop: its curves at `lo` and `hi`, and a wall under each.
fn ring(placed: &Placed, lp: &[Step], lo: f64, hi: f64) -> Result<Ring> {
    let curves: Vec<Cubic> = lp.iter().flat_map(|&s| oriented(placed, s)).collect();
    let n = curves.len();
    let lower: Vec<Vertex> = curves.iter().map(|c| Vertex::new(p3(c[0], lo))).collect();
    let upper: Vec<Vertex> = curves.iter().map(|c| Vertex::new(p3(c[0], hi))).collect();
    let uprights: Vec<Edge> = (0..n)
        .map(|i| {
            let line = Line(p3(curves[i][0], lo), p3(curves[i][0], hi));
            edge(&lower[i], &upper[i], Curve::Line(line))
        })
        .collect::<Result<_>>()?;
    let (mut bottom, mut top, mut walls) = (Wire::new(), Wire::new(), Vec::with_capacity(n));
    for (i, c) in curves.iter().enumerate() {
        let j = (i + 1) % n;
        let below = edge(&lower[i], &lower[j], curve(c, lo))?;
        let above = edge(&upper[i], &upper[j], curve(c, hi))?;
        let boundary = Wire::from(vec![
            below.clone(),
            uprights[j].clone(),
            above.inverse(),
            uprights[i].inverse(),
        ]);
        walls.push(
            Face::try_new(vec![boundary], wall(c, lo, hi))
                .map_err(|e| failed(format!("a STEP wall is invalid: {e:?}")))?,
        );
        bottom.push_back(below);
        top.push_back(above);
    }
    Ok(Ring { bottom, top, walls })
}

/// The flat plane at `z`, facing up or down.
fn plane(z: f64, up: bool) -> Surface {
    let (o, x, y) = (
        Point3::new(0.0, 0.0, z),
        Point3::new(1.0, 0.0, z),
        Point3::new(0.0, 1.0, z),
    );
    Surface::Plane(if up {
        Plane::new(o, x, y)
    } else {
        Plane::new(o, y, x)
    })
}

/// The area extruded from `lo` to `hi`.
fn body(placed: &Placed, area: &Area, lo: f64, hi: f64) -> Result<Body> {
    let mut faces = Vec::new();
    let (mut bottoms, mut tops) = (Vec::new(), Vec::new());
    for lp in std::iter::once(&area.outer).chain(&area.holes) {
        let r = ring(placed, lp, lo, hi)?;
        faces.extend(r.walls);
        bottoms.push(r.bottom.inverse());
        tops.push(r.top);
    }
    let flat = |wires: Vec<Wire>, up: bool| {
        Face::try_new(wires, plane(if up { hi } else { lo }, up))
            .map_err(|e| failed(format!("a flat STEP face is invalid: {e:?}")))
    };
    faces.push(flat(bottoms, false)?);
    faces.push(flat(tops, true)?);
    let solid = Solid::try_new(vec![Shell::from(faces)])
        .map_err(|e| failed(format!("a STEP body isn't a closed solid: {e:?}")))?;
    Ok(Body {
        solid,
        area: area.area,
        top: area.top,
        left: area.left,
    })
}

/// One solid per area of the slab.
pub fn solids(placed: &Placed, slab: &Slab) -> Result<Vec<Body>> {
    areas(placed, &slab.member)?
        .iter()
        .map(|area| body(placed, area, slab.lo, slab.hi))
        .collect()
}
