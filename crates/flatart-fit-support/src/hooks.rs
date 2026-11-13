//! Catching fits that double back.
//!
//! A fitting library can return a cubic with a tiny (or zero-length) handle
//! whose other handle points backwards (kurbo does). The curve runs back behind its start in a
//! U-turn a hair wide and comes back, all within the tolerance, so the
//! deviation check passes. Flattened into straight segments, the two sides
//! of that needle can cross, which the mesh can't triangulate. The same
//! happens at the far end when a curve overshoots its end and comes back,
//! and when a curve's handles cross so it loops over itself between its
//! ends: only fine flattening (large print sizes) resolves such a loop.

use flatart_domain::{CubicBezier, PxPoint};

type V = (f64, f64);

fn sub(a: PxPoint, b: PxPoint) -> V {
    (a.x.0 - b.x.0, a.y.0 - b.y.0)
}

fn dot(a: V, b: V) -> f64 {
    a.0 * b.0 + a.1 * b.1
}

fn first_long(vectors: [V; 3]) -> V {
    vectors
        .into_iter()
        .find(|v| v.0.hypot(v.1) > 1e-12)
        .unwrap_or((0.0, 0.0))
}

/// The way `c` sets off: along its first handle, or past it if it has no length.
fn start_direction(c: &CubicBezier) -> V {
    first_long([
        sub(c.ctrl1, c.from),
        sub(c.ctrl2, c.from),
        sub(c.to, c.from),
    ])
}

/// The way `c` arrives: along its last handle, or past it if it has no length.
fn end_direction(c: &CubicBezier) -> V {
    first_long([sub(c.to, c.ctrl2), sub(c.to, c.ctrl1), sub(c.to, c.from)])
}

/// Whether `c` stays between its ends along its chord: never behind its
/// start, never past its end. Exact: the position along the chord is a cubic
/// in t, checked at its turning points.
fn stays_between_its_ends(c: &CubicBezier) -> bool {
    let chord = sub(c.to, c.from);
    let length2 = dot(chord, chord);
    if length2 == 0.0 {
        return false;
    }
    // The position along the chord, as Bernstein coefficients (b0 = 0).
    let (b1, b2, b3) = (
        dot(sub(c.ctrl1, c.from), chord),
        dot(sub(c.ctrl2, c.from), chord),
        length2,
    );
    let at = |t: f64| {
        let u = 1.0 - t;
        3.0 * b1 * t * u * u + 3.0 * b2 * t * t * u + b3 * t * t * t
    };
    // Its derivative, a quadratic in t: a t² + b t + c.
    let (d0, d1, d2) = (b1, b2 - b1, b3 - b2);
    let (a, b, c0) = (d0 - 2.0 * d1 + d2, 2.0 * (d1 - d0), d0);
    let mut turning = Vec::with_capacity(2);
    if a.abs() > 1e-12 * length2 {
        let disc = b * b - 4.0 * a * c0;
        if disc >= 0.0 {
            let root = disc.sqrt();
            turning.extend([(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)]);
        }
    } else if b != 0.0 {
        turning.push(-c0 / b);
    }
    let slack = 1e-9 * length2;
    turning
        .into_iter()
        .filter(|t| (0.0..=1.0).contains(t))
        .map(at)
        .all(|f| f >= -slack && f <= length2 + slack)
}

/// Whether `c` crosses itself: a loop, which can stay between its ends and
/// within the tolerance, a hair wide. Exact: in power form B(t) = a t³ +
/// b t² + c t + d, B(s) = B(t) for s ≠ t means a (σ² − π) + b σ + c = 0,
/// with σ = s + t and π = s t, which gives σ, then π, then s and t.
fn crosses_itself(c: &CubicBezier) -> bool {
    let p = [c.from, c.ctrl1, c.ctrl2, c.to].map(|q| (q.x.0, q.y.0));
    let combine = |w: [f64; 4]| {
        (
            w[0] * p[0].0 + w[1] * p[1].0 + w[2] * p[2].0 + w[3] * p[3].0,
            w[0] * p[0].1 + w[1] * p[1].1 + w[2] * p[2].1 + w[3] * p[3].1,
        )
    };
    let a = combine([-1.0, 3.0, -3.0, 1.0]);
    let b = combine([3.0, -6.0, 3.0, 0.0]);
    let k = combine([-3.0, 3.0, 0.0, 0.0]);
    let cross = |u: V, v: V| u.0 * v.1 - u.1 * v.0;
    let denominator = cross(b, a);
    // a and b parallel (or a cubic that is really a quadratic): no loop.
    if denominator.abs() <= 1e-12 * a.0.hypot(a.1) * b.0.hypot(b.1) {
        return false;
    }
    let sigma = cross(a, k) / denominator;
    let (ai, bi, ki) = if a.0.abs() >= a.1.abs() {
        (a.0, b.0, k.0)
    } else {
        (a.1, b.1, k.1)
    };
    let pi = sigma * sigma + (bi * sigma + ki) / ai;
    let discriminant = sigma * sigma - 4.0 * pi;
    if discriminant <= 0.0 {
        return false;
    }
    let root = discriminant.sqrt();
    let (s, t) = ((sigma - root) / 2.0, (sigma + root) / 2.0);
    (0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&t)
}

/// Whether `curves`, fitted to the corner-free `run` (at least 2 points),
/// leave and reach its ends along it, each stay between their own ends
/// without crossing themselves, and never turn back where two of them meet.
pub fn none(curves: &[CubicBezier], run: &[PxPoint]) -> bool {
    let (Some(first), Some(last)) = (curves.first(), curves.last()) else {
        return true;
    };
    let n = run.len();
    dot(start_direction(first), sub(run[1], run[0])) > 0.0
        && dot(end_direction(last), sub(run[n - 1], run[n - 2])) > 0.0
        && curves
            .windows(2)
            .all(|w| dot(end_direction(&w[0]), start_direction(&w[1])) > 0.0)
        && curves
            .iter()
            .all(|c| stays_between_its_ends(c) && !crosses_itself(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(points: [(f64, f64); 4]) -> CubicBezier {
        let [a, b, c, d] = points.map(|(x, y)| PxPoint::new(x, y));
        CubicBezier {
            from: a,
            ctrl1: b,
            ctrl2: c,
            to: d,
        }
    }

    /// From a traced image: a 0.002 px first handle, then the second one
    /// behind the start. The curve runs 0.4 px back before heading on.
    #[test]
    fn a_curve_that_runs_back_behind_its_start_is_a_hook() {
        let c = curve([
            (495.5, 583.5),
            (495.502, 583.4998),
            (494.0765, 583.8547),
            (504.4009, 584.0568),
        ]);
        assert!(!stays_between_its_ends(&c));
    }

    /// The other end: the last handle reaches past the end, so the curve
    /// overshoots and comes back.
    #[test]
    fn a_curve_that_overshoots_its_end_is_a_hook() {
        let c = curve([
            (657.5, 366.25),
            (661.3439, 367.672),
            (661.0, 367.5),
            (661.0, 367.5),
        ]);
        assert!(!stays_between_its_ends(&c));
    }

    /// From Jolteon traced for 280 mm: the handles cross, so the curve
    /// loops (at t ≈ 0.56 and 0.65) while staying between its ends and
    /// within the tolerance. Flattened finely, the loop's sides cross.
    #[test]
    fn a_curve_that_loops_crosses_itself() {
        let c = curve([
            (676.1226, 425.1171),
            (674.9817, 428.7777),
            (676.5565, 422.774),
            (675.0, 429.0),
        ]);
        assert!(stays_between_its_ends(&c));
        assert!(crosses_itself(&c));
        assert!(!none(&[c], &[c.from, c.to]));
        // The textbook loop: handles reaching past each other.
        assert!(crosses_itself(&curve([
            (0.0, 0.0),
            (4.0, 2.0),
            (-1.0, 2.0),
            (3.0, 0.0)
        ])));
    }

    #[test]
    fn arcs_and_s_bends_dont_cross_themselves() {
        for c in [
            curve([(0.0, 0.0), (0.0, 1.33), (2.0, 1.33), (2.0, 0.0)]),
            curve([(0.0, 0.0), (2.0, 2.0), (1.0, -2.0), (3.0, 0.0)]),
            curve([(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)]),
            // A quadratic raised to a cubic: a and b are both zero or parallel.
            curve([(0.0, 0.0), (2.0, 4.0), (4.0, 4.0), (6.0, 0.0)]),
        ] {
            assert!(!crosses_itself(&c), "{c:?}");
        }
        // Crossed handles that still don't loop: the crossing falls outside
        // 0..1.
        assert!(!crosses_itself(&curve([
            (0.0, 0.0),
            (3.0, 1.0),
            (0.0, 1.0),
            (3.0, 0.0)
        ])));
    }

    #[test]
    fn lines_and_arcs_up_to_a_half_turn_are_fine() {
        assert!(stays_between_its_ends(&curve([
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0)
        ])));
        // About a half circle: the handles stand straight up from the ends.
        assert!(stays_between_its_ends(&curve([
            (0.0, 0.0),
            (0.0, 1.33),
            (2.0, 1.33),
            (2.0, 0.0)
        ])));
        // An S-bend.
        assert!(stays_between_its_ends(&curve([
            (0.0, 0.0),
            (2.0, 2.0),
            (1.0, -2.0),
            (3.0, 0.0)
        ])));
    }
}
