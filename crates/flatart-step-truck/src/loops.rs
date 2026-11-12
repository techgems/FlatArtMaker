//! The boundary of a set of areas, as closed loops of borders, and which
//! loops are holes in which.
//!
//! A set is given per border side: whether the area on its left, and on its
//! right, is in the set. The boundary is the borders whose two sides
//! differ. Each loop is walked with the set on its left, so in model space
//! (y up) an outer loop runs anticlockwise and a hole clockwise.

use crate::place::{Cubic, P2, Placed};
use flatart_domain::{Error, Result, Stage};
use std::collections::HashMap;

/// One border in a loop, possibly walked backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub border: usize,
    pub reversed: bool,
}

/// One area of the set: an outer loop and its holes.
#[derive(Debug, Clone)]
pub struct Area {
    pub outer: Vec<Step>,
    pub holes: Vec<Vec<Step>>,
    /// The area, holes taken out (mm²).
    pub area: f64,
    /// The outer loop's highest y and lowest x, for ordering.
    pub top: f64,
    pub left: f64,
}

fn failed(message: &str) -> Error {
    Error::StageFailed {
        stage: Stage::Export,
        message: message.into(),
    }
}

/// The step's curves in walking order, each oriented that way.
pub fn oriented(placed: &Placed, step: Step) -> Vec<Cubic> {
    let curves = &placed.curves[step.border];
    if step.reversed {
        curves
            .iter()
            .rev()
            .map(|c| [c[3], c[2], c[1], c[0]])
            .collect()
    } else {
        curves.clone()
    }
}

fn start_of(placed: &Placed, step: Step) -> u32 {
    let (start, end) = placed.ends[step.border];
    if step.reversed { end } else { start }
}

fn end_of(placed: &Placed, step: Step) -> u32 {
    let (start, end) = placed.ends[step.border];
    if step.reversed { start } else { end }
}

fn sub(a: P2, b: P2) -> P2 {
    (a.0 - b.0, a.1 - b.1)
}

fn nonzero(candidates: [P2; 3]) -> P2 {
    candidates
        .into_iter()
        .find(|v| v.0 != 0.0 || v.1 != 0.0)
        .unwrap_or((0.0, 0.0))
}

/// The direction a step sets off in, and arrives in.
fn leaving(placed: &Placed, step: Step) -> P2 {
    let c = oriented(placed, step)[0];
    nonzero([sub(c[1], c[0]), sub(c[2], c[0]), sub(c[3], c[0])])
}

fn arriving(placed: &Placed, step: Step) -> P2 {
    let curves = oriented(placed, step);
    let c = curves[curves.len() - 1];
    nonzero([sub(c[3], c[2]), sub(c[3], c[1]), sub(c[3], c[0])])
}

/// The set's boundary as closed loops. At a point where the set touches
/// itself, the walk turns as far left as it can, which keeps the parts that
/// meet there in separate loops.
pub fn loops(placed: &Placed, member: &[[bool; 2]]) -> Result<Vec<Vec<Step>>> {
    let steps: Vec<Step> = member
        .iter()
        .enumerate()
        .filter(|(_, m)| m[0] != m[1])
        .map(|(border, m)| Step {
            border,
            reversed: !m[0],
        })
        .collect();
    let mut from: HashMap<u32, Vec<Step>> = HashMap::new();
    for &step in &steps {
        from.entry(start_of(placed, step)).or_default().push(step);
    }
    let mut used = vec![false; placed.curves.len()];
    let mut out = Vec::new();
    for &first in &steps {
        if used[first.border] {
            continue;
        }
        let mut lp = Vec::new();
        let mut step = first;
        loop {
            used[step.border] = true;
            lp.push(step);
            if lp.len() > steps.len() {
                return Err(failed("a STEP loop doesn't close"));
            }
            let incoming = arriving(placed, step);
            let next = from
                .get(&end_of(placed, step))
                .into_iter()
                .flatten()
                .filter(|s| !used[s.border] || **s == first)
                .max_by(|a, b| {
                    let turn = |s: &Step| {
                        let out = leaving(placed, *s);
                        let cross = incoming.0 * out.1 - incoming.1 * out.0;
                        let dot = incoming.0 * out.0 + incoming.1 * out.1;
                        cross.atan2(dot)
                    };
                    turn(a).total_cmp(&turn(b))
                })
                .copied()
                .ok_or_else(|| failed("a STEP loop doesn't close"))?;
            if next == first {
                break;
            }
            step = next;
        }
        out.push(lp);
    }
    Ok(out)
}

/// Gauss–Legendre nodes and weights on 0..1: exact for the area integrand,
/// a degree-5 polynomial.
const GAUSS: [(f64, f64); 3] = [
    (0.112_701_665_379_258_3, 5.0 / 18.0),
    (0.5, 8.0 / 18.0),
    (0.887_298_334_620_741_7, 5.0 / 18.0),
];

fn point(c: &Cubic, t: f64) -> P2 {
    let s = 1.0 - t;
    let w = [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t];
    (0..4).fold((0.0, 0.0), |acc, i| {
        (acc.0 + w[i] * c[i].0, acc.1 + w[i] * c[i].1)
    })
}

fn derivative(c: &Cubic, t: f64) -> P2 {
    let s = 1.0 - t;
    let w = [3.0 * s * s, 6.0 * s * t, 3.0 * t * t];
    (0..3).fold((0.0, 0.0), |acc, i| {
        let d = sub(c[i + 1], c[i]);
        (acc.0 + w[i] * d.0, acc.1 + w[i] * d.1)
    })
}

/// The loop's signed area: positive when it runs anticlockwise.
pub fn signed_area(placed: &Placed, lp: &[Step]) -> f64 {
    lp.iter()
        .flat_map(|&s| oriented(placed, s))
        .map(|c| {
            GAUSS
                .iter()
                .map(|&(t, w)| {
                    let (p, d) = (point(&c, t), derivative(&c, t));
                    w * (p.0 * d.1 - p.1 * d.0) / 2.0
                })
                .sum::<f64>()
        })
        .sum()
}

/// The loop as a polyline, for inside tests.
fn polyline(placed: &Placed, lp: &[Step]) -> Vec<P2> {
    const SAMPLES: usize = 16;
    lp.iter()
        .flat_map(|&s| oriented(placed, s))
        .flat_map(|c| (0..SAMPLES).map(move |k| point(&c, k as f64 / SAMPLES as f64)))
        .collect()
}

/// Even-odd test of `p` against `ring`.
fn inside(p: P2, ring: &[P2]) -> bool {
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

/// The set's areas: each outer loop with the holes inside it.
pub fn areas(placed: &Placed, member: &[[bool; 2]]) -> Result<Vec<Area>> {
    let all = loops(placed, member)?;
    let signed: Vec<f64> = all.iter().map(|lp| signed_area(placed, lp)).collect();
    let outers: Vec<usize> = (0..all.len()).filter(|&i| signed[i] > 0.0).collect();
    let rings: Vec<Vec<P2>> = outers.iter().map(|&i| polyline(placed, &all[i])).collect();
    let mut areas: Vec<Area> = outers
        .iter()
        .zip(&rings)
        .map(|(&i, ring)| Area {
            outer: all[i].clone(),
            holes: Vec::new(),
            area: signed[i],
            top: ring.iter().map(|p| p.1).fold(f64::MIN, f64::max),
            left: ring.iter().map(|p| p.0).fold(f64::MAX, f64::min),
        })
        .collect();
    for hole in (0..all.len()).filter(|&i| signed[i] <= 0.0) {
        // A point on the hole away from its ends, which may touch the outer loop.
        let probe = point(&oriented(placed, all[hole][0])[0], 0.5);
        let around = (0..outers.len())
            .filter(|&k| inside(probe, &rings[k]))
            .min_by(|&x, &y| signed[outers[x]].total_cmp(&signed[outers[y]]))
            .ok_or_else(|| failed("a hole in a STEP face isn't inside any outline"))?;
        areas[around].holes.push(all[hole].clone());
        areas[around].area += signed[hole];
    }
    Ok(areas)
}
