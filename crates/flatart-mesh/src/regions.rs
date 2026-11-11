//! The areas to fill, as rings of outline points.
//!
//! Every ring walks with its region on the right, as the border map's loops
//! do. Regions are the faces, in order, then the outside areas the art
//! encloses, which the base fills when asked.
//!
//! A region can meet itself at a junction (two of its cells touching only at
//! a corner). Each time a ring passes a junction is a separate *corner*, so
//! the surfaces on either side of such a point get separate vertices.

use crate::flatten::Outline;
use flatart_domain::{BorderMap, Error, Loop, Result, Stage};
use std::collections::HashMap;

/// No corner: a point that isn't a junction.
pub const NO_CORNER: u32 = u32::MAX;

/// A point of a ring, and the corner it is when it's a junction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RingPoint {
    pub point: u32,
    pub corner: u32,
}

/// A border walked one way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Step {
    border: usize,
    reversed: bool,
}

pub struct Region {
    /// The outer ring, then the holes.
    pub rings: Vec<Vec<RingPoint>>,
}

pub struct Regions {
    pub regions: Vec<Region>,
    /// How many of `regions` are faces (the rest are enclosed outside areas).
    pub faces: usize,
    /// The region on each border's left and right, if any.
    pub sides: Vec<[Option<u32>; 2]>,
    /// The corner where `region`'s ring passes the start (false) or end
    /// (true) of `border`.
    corners: HashMap<(u32, u32, bool), u32>,
}

impl Regions {
    /// The corner of `region` at point `index` along `border`.
    pub fn corner(&self, outline: &Outline, region: u32, border: usize, index: usize) -> u32 {
        let ids = &outline.borders[border];
        let point = ids[index] as usize;
        if !outline.junction[point] {
            return NO_CORNER;
        }
        let end = index + 1 == ids.len();
        if index != 0 && !end {
            return NO_CORNER;
        }
        self.corners
            .get(&(region, border as u32, end))
            .copied()
            .unwrap_or(NO_CORNER)
    }
}

fn failed(message: &str) -> Error {
    Error::StageFailed {
        stage: Stage::Mesh,
        message: message.into(),
    }
}

struct Builder<'o> {
    outline: &'o Outline,
    regions: Vec<Region>,
    sides: Vec<[Option<u32>; 2]>,
    corners: HashMap<(u32, u32, bool), u32>,
    next_corner: u32,
}

impl Builder<'_> {
    /// The points of `steps` as one ring, recording its corners and sides
    /// for `region`.
    fn ring(&mut self, region: u32, steps: &[Step]) -> Vec<RingPoint> {
        let mut ring = Vec::new();
        for (i, step) in steps.iter().enumerate() {
            let ids = &self.outline.borders[step.border];
            let mut ids: Vec<u32> = ids.clone();
            if step.reversed {
                ids.reverse();
            }
            // Where this step starts: a corner if it's a junction.
            let start = ids[0];
            let corner = if self.outline.junction[start as usize] {
                let c = self.next_corner;
                self.next_corner += 1;
                let prev = steps[(i + steps.len() - 1) % steps.len()];
                // The previous step ends here, this one starts here.
                self.corners
                    .insert((region, prev.border as u32, !prev.reversed), c);
                self.corners
                    .insert((region, step.border as u32, step.reversed), c);
                c
            } else {
                NO_CORNER
            };
            ring.push(RingPoint {
                point: start,
                corner,
            });
            ring.extend(ids[1..ids.len() - 1].iter().map(|&point| RingPoint {
                point,
                corner: NO_CORNER,
            }));
            self.sides[step.border][usize::from(!step.reversed)] = Some(region);
        }
        ring
    }
}

fn steps(lp: &Loop) -> Vec<Step> {
    lp.steps
        .iter()
        .map(|s| Step {
            border: s.border.0 as usize,
            reversed: s.reversed,
        })
        .collect()
}

/// Loops of the borders that touch outside, each walked with outside on its
/// right. At a junction with a choice, the walk turns as far right as it
/// can, which keeps outside areas that touch only at a corner apart.
fn outside_loops(map: &BorderMap, outline: &Outline) -> Result<Vec<Vec<Step>>> {
    let mut leaving: HashMap<u32, Vec<Step>> = HashMap::new();
    let mut walks = Vec::new();
    for (border, b) in map.borders.iter().enumerate() {
        let reversed = match (b.left.is_outside(), b.right.is_outside()) {
            (false, true) => false,
            (true, false) => true,
            _ => continue,
        };
        let step = Step { border, reversed };
        leaving
            .entry(start_of(outline, step))
            .or_default()
            .push(step);
        walks.push(step);
    }
    let point = |id: u32| outline.points[id as usize];
    let mut used = vec![false; map.borders.len()];
    let mut loops = Vec::new();
    for &first in &walks {
        if used[first.border] {
            continue;
        }
        let mut lp = Vec::new();
        let mut step = first;
        loop {
            used[step.border] = true;
            lp.push(step);
            let ids = ordered(outline, step);
            let (at, before) = (ids[ids.len() - 1], ids[ids.len() - 2]);
            let incoming = sub(point(at), point(before));
            let next = leaving
                .get(&at)
                .into_iter()
                .flatten()
                .filter(|s| !used[s.border] || **s == first)
                .min_by(|a, b| {
                    let turn = |s: &Step| {
                        let ids = ordered(outline, *s);
                        let out = sub(point(ids[1]), point(at));
                        cross(incoming, out).atan2(dot(incoming, out))
                    };
                    turn(a).total_cmp(&turn(b))
                })
                .copied()
                .ok_or_else(|| failed("an outside loop doesn't close"))?;
            if next == first {
                break;
            }
            step = next;
        }
        loops.push(lp);
    }
    Ok(loops)
}

fn start_of(outline: &Outline, step: Step) -> u32 {
    let ids = &outline.borders[step.border];
    if step.reversed {
        ids[ids.len() - 1]
    } else {
        ids[0]
    }
}

fn ordered(outline: &Outline, step: Step) -> Vec<u32> {
    let mut ids = outline.borders[step.border].clone();
    if step.reversed {
        ids.reverse();
    }
    ids
}

fn sub(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 - b.0, a.1 - b.1)
}

fn cross(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.1 - a.1 * b.0
}

fn dot(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1
}

/// Even-odd test of `p` against `ring`.
fn inside(p: (f64, f64), ring: &[(f64, f64)]) -> bool {
    let n = ring.len();
    let mut result = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + n - 1) % n]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            result = !result;
        }
    }
    result
}

/// The faces, then the enclosed outside areas.
pub fn build(map: &BorderMap, outline: &Outline) -> Result<Regions> {
    let mut b = Builder {
        outline,
        regions: Vec::with_capacity(map.faces.len()),
        sides: vec![[None, None]; map.borders.len()],
        corners: HashMap::new(),
        next_corner: 0,
    };
    for (f, face) in map.faces.iter().enumerate() {
        let region = f as u32;
        let mut rings = vec![b.ring(region, &steps(&face.outer))];
        for hole in &face.holes {
            rings.push(b.ring(region, &steps(hole)));
        }
        b.regions.push(Region { rings });
    }
    let faces = b.regions.len();

    // Outside loops with outside on the right: clockwise (negative, y up)
    // round an enclosed area, anticlockwise round a piece of art.
    let loops = outside_loops(map, outline)?;
    let ring_points = |lp: &[Step]| -> Vec<(f64, f64)> {
        lp.iter()
            .flat_map(|s| {
                let ids = ordered(outline, *s);
                ids[..ids.len() - 1]
                    .iter()
                    .map(|&i| outline.points[i as usize])
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    let shapes: Vec<Vec<(f64, f64)>> = loops.iter().map(|lp| ring_points(lp)).collect();
    let area = |ring: &[(f64, f64)]| -> f64 {
        let n = ring.len();
        (0..n)
            .map(|i| cross(ring[i], ring[(i + 1) % n]))
            .sum::<f64>()
            / 2.0
    };
    let enclosed: Vec<usize> = (0..loops.len())
        .filter(|&i| area(&shapes[i]) < 0.0)
        .collect();
    let mut holes: Vec<Vec<usize>> = vec![Vec::new(); enclosed.len()];
    for i in (0..loops.len()).filter(|&i| area(&shapes[i]) >= 0.0) {
        let probe = {
            let (a, c) = (shapes[i][0], shapes[i][1 % shapes[i].len()]);
            ((a.0 + c.0) / 2.0, (a.1 + c.1) / 2.0)
        };
        let around = enclosed
            .iter()
            .enumerate()
            .filter(|&(_, &e)| inside(probe, &shapes[e]))
            .min_by(|x, y| {
                area(&shapes[*x.1])
                    .abs()
                    .total_cmp(&area(&shapes[*y.1]).abs())
            });
        if let Some((k, _)) = around {
            holes[k].push(i);
        }
    }
    for (k, &e) in enclosed.iter().enumerate() {
        let region = b.regions.len() as u32;
        let mut rings = vec![b.ring(region, &loops[e])];
        for &h in &holes[k] {
            rings.push(b.ring(region, &loops[h]));
        }
        b.regions.push(Region { rings });
    }
    Ok(Regions {
        regions: b.regions,
        faces,
        sides: b.sides,
        corners: b.corners,
    })
}
