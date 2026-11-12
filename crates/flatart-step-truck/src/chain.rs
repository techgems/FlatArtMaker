//! Joining a loop's curves into as few edges as the shape allows.
//!
//! The fitter gives many short curves and lines, nearly all of them
//! meeting tangentially. A run of curves (lines included) that meet
//! smoothly is one cubic B-spline, exactly: each curve gets a parameter
//! span in proportion to its handles at the joins, so the spline is C1
//! there (each interior knot appears twice) and never stops moving.
//!
//! A run breaks, leaving a vertex, wherever that isn't possible:
//! - at a corner, and at the end of each border;
//! - where a handle at the join has no length (the spline would stop
//!   there);
//! - where a curve's span would drift too far from its length.
//!
//! Nothing less smooth is kept inside an edge: a point where the parameter
//! stops, or a C0 knot, inside an edge (and so inside the wall under it)
//! is where CAD kernels fail to mesh or check a face. A run of nothing but
//! lines stays a line (or lines), so its wall stays a plane.

use crate::loops::{Step, oriented};
use crate::place::{Cubic, P2, Placed};

/// How far a join may move (mm) when the spline is made C1 there. Handles
/// a hair off parallel move it by about their length times the angle.
const SNUG: f64 = 1e-7;

/// How far a curve's parameter span may drift from its chord length (as a
/// factor either way) before the run breaks. C1 fixes only the ratio of
/// neighbouring spans, so along a long run the ratios multiply, and a span
/// could shrink until readers (truck among them) take its knots for one.
const DRIFT: f64 = 10.0;

/// One edge's worth of a loop.
#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    /// A straight line from the first point to the second.
    Line(P2, P2),
    /// A cubic B-spline: its knots (in mm along it; each interior one
    /// twice, both ends four times) and control points.
    Spline { knots: Vec<f64>, points: Vec<P2> },
}

impl Piece {
    pub fn start(&self) -> P2 {
        match self {
            Piece::Line(a, _) => *a,
            Piece::Spline { points, .. } => points[0],
        }
    }
}

fn sub(a: P2, b: P2) -> P2 {
    (a.0 - b.0, a.1 - b.1)
}

fn len(v: P2) -> f64 {
    v.0.hypot(v.1)
}

fn chord(c: &Cubic) -> f64 {
    len(sub(c[3], c[0]))
}

/// Whether `b` carries on smoothly from `a`: both handles at the join have
/// length and point the same way, near enough that making the spline C1
/// there moves the join by at most `SNUG`.
fn smooth(a: &Cubic, b: &Cubic) -> bool {
    let (u, v) = (sub(a[3], a[2]), sub(b[1], b[0]));
    let (lu, lv) = (len(u), len(v));
    if lu == 0.0 || lv == 0.0 {
        return false;
    }
    let sine = (u.0 * v.1 - u.1 * v.0).abs() / (lu * lv);
    u.0 * v.0 + u.1 * v.1 > 0.0 && lu.min(lv) * sine <= SNUG
}

/// Whether the curve is a straight line from its start to its end.
pub fn straight(c: &Cubic) -> bool {
    let d = sub(c[3], c[0]);
    let len2 = d.0 * d.0 + d.1 * d.1;
    let off = |p: P2| ((p.0 - c[0].0) * d.1 - (p.1 - c[0].1) * d.0).abs();
    len2 > 0.0 && off(c[1]) <= 1e-12 * len2 && off(c[2]) <= 1e-12 * len2
}

/// One run of smoothly joined curves as pieces.
fn to_pieces(run: &[Cubic]) -> Vec<Piece> {
    if run.iter().all(straight) {
        return lines(run);
    }
    let spans = spans(run);
    // Never the first curve: its span is its chord.
    let drifted = spans
        .iter()
        .zip(run)
        .position(|(s, c)| !(1.0 / DRIFT..=DRIFT).contains(&(s / chord(c))));
    match drifted {
        Some(i) => {
            let mut out = to_pieces(&run[..i]);
            out.extend(to_pieces(&run[i..]));
            out
        }
        None => vec![spline(run, &spans)],
    }
}

/// Lines only: one line if every join lies on it, else each on its own.
fn lines(run: &[Cubic]) -> Vec<Piece> {
    let (a, b) = (run[0][0], run[run.len() - 1][3]);
    let d = sub(b, a);
    let on_line = |p: P2| {
        let q = sub(p, a);
        (q.0 * d.1 - q.1 * d.0).abs() / len(d) <= SNUG
    };
    if run.len() > 1 && len(d) > 0.0 && run[..run.len() - 1].iter().all(|c| on_line(c[3])) {
        return vec![Piece::Line(a, b)];
    }
    run.iter().map(|c| Piece::Line(c[0], c[3])).collect()
}

/// Each curve's parameter span, making the spline C1 at every join:
/// (a3 - a2) / h_a = (b1 - b0) / h_b. The first span is its chord, so the
/// knots are in mm along the run, near the curves' lengths.
fn spans(run: &[Cubic]) -> Vec<f64> {
    let mut spans = vec![chord(&run[0])];
    for w in run.windows(2) {
        let last = spans[spans.len() - 1];
        spans.push(last * len(sub(w[1][1], w[1][0])) / len(sub(w[0][3], w[0][2])));
    }
    spans
}

/// One B-spline through a run of smoothly joined curves.
fn spline(run: &[Cubic], spans: &[f64]) -> Piece {
    let mut knots = vec![0.0; 4];
    let mut points = vec![run[0][0], run[0][1], run[0][2]];
    let mut at = 0.0;
    for (span, c) in spans.iter().zip(&run[1..]) {
        at += span;
        knots.extend([at, at]);
        points.extend([c[1], c[2]]);
    }
    knots.extend([at + spans[spans.len() - 1]; 4]);
    points.push(run[run.len() - 1][3]);
    Piece::Spline { knots, points }
}

/// The loop's pieces, in walking order. Every piece starts where the last
/// one ends, and the last ends where the first starts.
pub fn pieces(placed: &Placed, lp: &[Step]) -> Vec<Piece> {
    let mut runs: Vec<Vec<Cubic>> = Vec::new();
    for &step in lp {
        let mut run: Vec<Cubic> = Vec::new();
        for c in oriented(placed, step) {
            if run.last().is_some_and(|last| !smooth(last, &c)) {
                runs.push(std::mem::take(&mut run));
            }
            run.push(c);
        }
        runs.push(run);
    }
    let mut out: Vec<Piece> = runs.iter().flat_map(|r| to_pieces(r)).collect();
    // One piece round the whole loop would start where it ends: split it.
    // A closed loop has at least two curves.
    if out.len() == 1
        && let [run] = runs.as_slice()
    {
        let mid = run.len() / 2;
        out = to_pieces(&run[..mid]);
        out.extend(to_pieces(&run[mid..]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use truck_modeling::{BSplineCurve, KnotVec, ParametricCurve, Point3};

    fn bezier(c: &Cubic, s: f64) -> P2 {
        let r = 1.0 - s;
        let w = [r * r * r, 3.0 * r * r * s, 3.0 * r * s * s, s * s * s];
        (0..4).fold((0.0, 0.0), |a, i| {
            (a.0 + w[i] * c[i].0, a.1 + w[i] * c[i].1)
        })
    }

    /// Three tangent cubics with handles of different lengths at each join.
    fn run() -> Vec<Cubic> {
        let a = [(0.0, 0.0), (1.0, 2.0), (3.0, 3.0), (5.0, 3.0)];
        // Carries on along (1, 0), with a handle 3 times as long.
        let b = [(5.0, 3.0), (11.0, 3.0), (12.0, -1.0), (10.0, -4.0)];
        // Carries on along (-2, -3), with a handle a fifth as long.
        let c = [(10.0, -4.0), (9.6, -4.6), (7.0, -6.0), (4.0, -5.0)];
        vec![a, b, c]
    }

    #[test]
    fn a_tangent_run_is_one_spline_with_the_same_shape() {
        let run = run();
        assert!(run.windows(2).all(|w| smooth(&w[0], &w[1])));
        let [Piece::Spline { knots, points }] = <[Piece; 1]>::try_from(to_pieces(&run)).unwrap()
        else {
            panic!("not one spline")
        };
        assert_eq!(points.len(), 2 * run.len() + 2);
        let curve = BSplineCurve::new(
            KnotVec::from(knots.clone()),
            points.iter().map(|p| Point3::new(p.0, p.1, 0.0)).collect(),
        );
        // Span i runs between knots[3 + 2i] and knots[4 + 2i].
        let mut worst: f64 = 0.0;
        for (i, c) in run.iter().enumerate() {
            let (t0, t1) = (knots[3 + 2 * i], knots[4 + 2 * i]);
            for k in 0..=20 {
                let s = f64::from(k) / 20.0;
                let p = curve.subs(t0 + s * (t1 - t0));
                let q = bezier(c, s);
                worst = worst.max((p.x - q.0).hypot(p.y - q.1));
            }
        }
        assert!(worst < 1e-12, "the spline strays {worst} from the curves");
    }

    #[test]
    fn a_corner_or_a_handle_of_no_length_breaks_the_run() {
        let [a, b, _] = <[Cubic; 3]>::try_from(run()).unwrap();
        let kinked = [b[0], (b[1].0, b[1].1 + 0.5), b[2], b[3]];
        assert!(!smooth(&a, &kinked));
        let back = [b[0], (2.0 * b[0].0 - b[1].0, b[0].1), b[2], b[3]];
        assert!(!smooth(&a, &back));
        // Tangent, but the spline would stop at the join.
        let still = [b[0], b[0], b[2], b[3]];
        assert!(!smooth(&a, &still));
        let line = [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)];
        assert!(straight(&line));
        assert!(!straight(&a));
    }

    /// The largest distance from a point on the curves to the same point
    /// on the pieces (spline span k is curve k; a line covers its curves).
    fn trace(run: &[Cubic], pieces: &[Piece]) -> f64 {
        let mut worst: f64 = 0.0;
        let mut k = 0;
        for piece in pieces {
            match piece {
                Piece::Spline { knots, points } => {
                    let curve = BSplineCurve::new(
                        KnotVec::from(knots.clone()),
                        points.iter().map(|p| Point3::new(p.0, p.1, 0.0)).collect(),
                    );
                    let mut ends: Vec<f64> = knots.clone();
                    ends.dedup();
                    for w in ends.windows(2) {
                        for i in 0..=20 {
                            let s = f64::from(i) / 20.0;
                            let p = curve.subs(w[0] + s * (w[1] - w[0]));
                            let q = bezier(&run[k], s);
                            worst = worst.max((p.x - q.0).hypot(p.y - q.1));
                        }
                        k += 1;
                    }
                }
                Piece::Line(a, b) => {
                    let d = sub(*b, *a);
                    while k < run.len() && straight(&run[k]) {
                        for i in 0..=20 {
                            let q = sub(bezier(&run[k], f64::from(i) / 20.0), *a);
                            let t =
                                ((q.0 * d.0 + q.1 * d.1) / (d.0 * d.0 + d.1 * d.1)).clamp(0.0, 1.0);
                            worst = worst.max((q.0 - t * d.0).hypot(q.1 - t * d.1));
                        }
                        k += 1;
                        if run[k - 1][3] == *b {
                            break;
                        }
                    }
                }
            }
        }
        assert_eq!(k, run.len(), "the pieces cover {k} of {} curves", run.len());
        worst
    }

    /// A small deterministic generator.
    struct Lcg(u64);

    impl Lcg {
        fn unit(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    /// Runs like the fitter's: lines and curves, each setting off along the
    /// last one's end tangent, with handles from very short to long, some
    /// of no length.
    #[test]
    fn random_tangent_runs_keep_their_shape() {
        let mut rng = Lcg(5);
        let (mut splines, mut drifted) = (0, 0);
        for case in 0..500 {
            let mut at = (rng.unit() * 100.0, rng.unit() * 100.0);
            let mut dir = (1.0_f64, 0.0_f64);
            let mut curves: Vec<Cubic> = Vec::new();
            for _ in 0..(1 + (rng.unit() * 30.0) as usize) {
                let step = 0.05 + rng.unit() * 5.0;
                let c = if rng.unit() < 0.4 {
                    let e = (at.0 + dir.0 * step, at.1 + dir.1 * step);
                    let third = |t: f64| (at.0 + (e.0 - at.0) * t, at.1 + (e.1 - at.1) * t);
                    [at, third(1.0 / 3.0), third(2.0 / 3.0), e]
                } else {
                    // Handles from 1e-4 of the step up to the step; now and
                    // then one of no length.
                    let h = step * 10f64.powf(-4.0 * rng.unit());
                    let zero = rng.unit() < 0.1;
                    let c1 = if zero {
                        at
                    } else {
                        (at.0 + dir.0 * h, at.1 + dir.1 * h)
                    };
                    let turn = (rng.unit() - 0.5) * 2.0;
                    let nd = (
                        dir.0 * turn.cos() - dir.1 * turn.sin(),
                        dir.0 * turn.sin() + dir.1 * turn.cos(),
                    );
                    let e = (at.0 + nd.0 * step, at.1 + nd.1 * step);
                    let back = step * 10f64.powf(-3.0 * rng.unit());
                    let c2 = (e.0 - nd.0 * back, e.1 - nd.1 * back);
                    // With no start handle, the direction is towards c2.
                    let c2 = if zero {
                        (at.0 + dir.0 * step * 0.5, at.1 + dir.1 * step * 0.5)
                    } else {
                        c2
                    };
                    let c = [at, c1, c2, e];
                    let u = sub(c[3], c[2]);
                    let l = len(u);
                    if l > 0.0 {
                        dir = (u.0 / l, u.1 / l);
                    }
                    c
                };
                at = c[3];
                curves.push(c);
            }
            // Split where pieces() would, then check each run.
            let mut runs: Vec<Vec<Cubic>> = vec![Vec::new()];
            for c in curves {
                if runs[runs.len() - 1].last().is_some_and(|l| !smooth(l, &c)) {
                    runs.push(Vec::new());
                }
                let last = runs.len() - 1;
                runs[last].push(c);
            }
            for run in runs {
                let pieces = to_pieces(&run);
                let worst = trace(&run, &pieces);
                assert!(worst < 1e-6, "case {case}: the pieces stray {worst} mm");
                for piece in &pieces {
                    let Piece::Spline { knots, points } = piece else {
                        continue;
                    };
                    splines += 1;
                    // C1 inside: no knot three times, and no control point
                    // twice in a row at a join (where the spline would
                    // stop). The ends are vertices, as before joining.
                    let n = points.len();
                    assert!(knots[4..knots.len() - 4].windows(3).all(|w| w[0] != w[2]));
                    assert!(points[1..n - 1].windows(2).all(|w| w[0] != w[1]));
                }
                drifted += usize::from(
                    pieces.len() > 1 && pieces.iter().any(|p| matches!(p, Piece::Spline { .. })),
                );
            }
        }
        assert!(
            splines > 500 && drifted > 20,
            "only {splines} splines, {drifted} runs broken for drift"
        );
    }
}
