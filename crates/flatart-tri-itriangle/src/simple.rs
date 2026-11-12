//! Checking the rings are simple before i_triangle sees them.
//!
//! i_triangle's unchecked triangulation trusts its input: on rings that
//! cross it indexes out of bounds, which is undefined behaviour (a crash in
//! a release build). So the rings are checked on the same integer grid, with
//! exact arithmetic, and a bad polygon is an error instead.

use i_triangle::i_overlay::i_float::int::point::IntPoint;
use std::collections::HashMap;

type Segment = (IntPoint<i32>, IntPoint<i32>);

/// The sign of the turn a → b → c: positive anticlockwise, 0 if collinear.
/// Exact: coordinates are within ±2²⁷, so the products fit in an i64.
fn orient(a: IntPoint<i32>, b: IntPoint<i32>, c: IntPoint<i32>) -> i64 {
    let (abx, aby) = (
        i64::from(b.x) - i64::from(a.x),
        i64::from(b.y) - i64::from(a.y),
    );
    let (acx, acy) = (
        i64::from(c.x) - i64::from(a.x),
        i64::from(c.y) - i64::from(a.y),
    );
    (abx * acy - aby * acx).signum()
}

/// Whether `p`, collinear with `a`–`b`, lies strictly between them.
fn strictly_inside(p: IntPoint<i32>, (a, b): Segment) -> bool {
    let between = |v: i32, s: i32, t: i32| s.min(t) <= v && v <= s.max(t);
    p != a && p != b && between(p.x, a.x, b.x) && between(p.y, a.y, b.y)
}

/// Why two edges can't both be in a simple polygon, if they can't.
fn conflict(s: Segment, t: Segment) -> Option<&'static str> {
    let (a, b) = s;
    let (c, d) = t;
    if s == t || s == (d, c) {
        return Some("two edges overlap");
    }
    let (o1, o2) = (orient(a, b, c), orient(a, b, d));
    let (o3, o4) = (orient(c, d, a), orient(c, d, b));
    if o1 * o2 < 0 && o3 * o4 < 0 {
        return Some("two edges cross");
    }
    let touches = (o1 == 0 && strictly_inside(c, s))
        || (o2 == 0 && strictly_inside(d, s))
        || (o3 == 0 && strictly_inside(a, t))
        || (o4 == 0 && strictly_inside(b, t));
    touches.then_some("a point lies on another edge")
}

/// Checks that no ring crosses itself or another, no point lies on an edge
/// it isn't an end of, and no edge has zero length. Rings may share points.
pub fn check(rings: &[Vec<IntPoint<i32>>]) -> Result<(), &'static str> {
    let mut segments: Vec<Segment> = Vec::new();
    for ring in rings {
        for (i, &a) in ring.iter().enumerate() {
            let b = ring[(i + 1) % ring.len()];
            if a == b {
                return Err("an edge has no length");
            }
            segments.push((a, b));
        }
    }
    // Bucket the edges on a grid about √n cells across, so only edges near
    // each other are compared.
    let (mut lo, mut hi) = ((i64::MAX, i64::MAX), (i64::MIN, i64::MIN));
    for &(a, b) in &segments {
        for p in [a, b] {
            lo = (lo.0.min(i64::from(p.x)), lo.1.min(i64::from(p.y)));
            hi = (hi.0.max(i64::from(p.x)), hi.1.max(i64::from(p.y)));
        }
    }
    let across = (segments.len() as f64).sqrt().ceil().max(1.0) as i64;
    let cell = ((hi.0 - lo.0).max(hi.1 - lo.1) / across).max(1) + 1;
    let at = |v: i32, origin: i64| (i64::from(v) - origin) / cell;
    let mut buckets: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, &(a, b)) in segments.iter().enumerate() {
        for x in at(a.x.min(b.x), lo.0)..=at(a.x.max(b.x), lo.0) {
            for y in at(a.y.min(b.y), lo.1)..=at(a.y.max(b.y), lo.1) {
                buckets.entry((x, y)).or_default().push(i);
            }
        }
    }
    for members in buckets.values() {
        for (k, &i) in members.iter().enumerate() {
            for &j in &members[k + 1..] {
                if let Some(why) = conflict(segments[i], segments[j]) {
                    return Err(why);
                }
            }
        }
    }
    Ok(())
}
