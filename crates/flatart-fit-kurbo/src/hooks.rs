//! Catching fits that double back.
//!
//! kurbo sometimes returns a cubic with a tiny (or zero-length) handle whose
//! other handle points backwards. The curve runs back behind its start in a
//! U-turn a hair wide and comes back, all within the tolerance, so the
//! deviation check passes. Flattened into straight segments, the two sides
//! of that needle can cross, which the mesh can't triangulate. The same
//! happens at the far end when a curve overshoots its end and comes back.

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

/// Whether `curves`, fitted to the corner-free `run` (at least 2 points),
/// leave and reach its ends along it, each stay between their own ends, and
/// never turn back where two of them meet.
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
        && curves.iter().all(stays_between_its_ends)
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
