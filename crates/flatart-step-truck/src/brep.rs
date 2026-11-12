//! Closed solids from a slab, with truck: one straight extrusion per area.
//!
//! An area's loops give its flat bottom and top. Each piece along them (a
//! line, or a run of tangent curves as one B-spline; see `chain`) gives one
//! vertical wall, and walls meet at vertical lines through the pieces'
//! ends. Every edge is exact. Each loop gets its own vertices, so areas that touch at
//! a point, or an area that touches itself, still give closed manifold
//! solids.

use crate::chain::{Piece, pieces};
use crate::loops::{Area, Step, areas};
use crate::parts::Slab;
use crate::place::{P2, Placed};
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

fn knots(knots: &[f64]) -> KnotVec {
    KnotVec::from(knots.to_vec())
}

fn curve(piece: &Piece, z: f64) -> Curve {
    match piece {
        Piece::Line(a, b) => Curve::Line(Line(p3(*a, z), p3(*b, z))),
        Piece::Spline { knots: k, points } => Curve::BSplineCurve(BSplineCurve::new(
            knots(k),
            points.iter().map(|&p| p3(p, z)).collect(),
        )),
    }
}

/// The vertical wall under `piece` from `lo` to `hi`, facing the piece's
/// right (the material is on its left).
fn wall(piece: &Piece, lo: f64, hi: f64) -> Surface {
    match piece {
        Piece::Line(a, b) => Surface::Plane(Plane::new(p3(*a, lo), p3(*b, lo), p3(*a, hi))),
        Piece::Spline { knots: k, points } => Surface::BSplineSurface(BSplineSurface::new(
            (knots(k), KnotVec::bezier_knot(1)),
            points.iter().map(|&p| vec![p3(p, lo), p3(p, hi)]).collect(),
        )),
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

/// Builds one loop: its pieces at `lo` and `hi`, and a wall under each.
fn ring(placed: &Placed, lp: &[Step], lo: f64, hi: f64) -> Result<Ring> {
    let pieces = pieces(placed, lp);
    let n = pieces.len();
    let lower: Vec<Vertex> = pieces
        .iter()
        .map(|c| Vertex::new(p3(c.start(), lo)))
        .collect();
    let upper: Vec<Vertex> = pieces
        .iter()
        .map(|c| Vertex::new(p3(c.start(), hi)))
        .collect();
    let uprights: Vec<Edge> = (0..n)
        .map(|i| {
            let line = Line(p3(pieces[i].start(), lo), p3(pieces[i].start(), hi));
            edge(&lower[i], &upper[i], Curve::Line(line))
        })
        .collect::<Result<_>>()?;
    let (mut bottom, mut top, mut walls) = (Wire::new(), Wire::new(), Vec::with_capacity(n));
    for (i, c) in pieces.iter().enumerate() {
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
