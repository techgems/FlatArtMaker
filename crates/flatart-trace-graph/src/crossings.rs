//! Finding borders whose curves cross.
//!
//! Each curve is sampled into short segments, which are bucketed on a grid so
//! only nearby segments are compared. Touching (at a shared junction, or
//! where one curve joins the next) isn't crossing.

use flatart_domain::CubicBezier;
use std::collections::{BTreeSet, HashMap};

/// Points sampled along each curve: the same as the conformance suite, so a
/// map this check passes, the suite passes too.
const SAMPLES_PER_CURVE: usize = 16;

/// The side of a bucket, in source pixels.
const BUCKET: f64 = 1.0;

type P = (f64, f64);

struct Segment {
    border: usize,
    a: P,
    b: P,
}

fn sample(curves: &[CubicBezier]) -> Vec<P> {
    let mut points = Vec::with_capacity(curves.len() * SAMPLES_PER_CURVE + 1);
    for (i, curve) in curves.iter().enumerate() {
        let first = usize::from(i > 0);
        points.extend((first..=SAMPLES_PER_CURVE).map(|k| {
            let p = curve.point_at(k as f64 / SAMPLES_PER_CURVE as f64);
            (p.x.0, p.y.0)
        }));
    }
    points
}

fn cross(o: P, a: P, b: P) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

/// Whether the segments cross at a point inside both.
fn segments_cross(s: &Segment, t: &Segment) -> bool {
    let eps = 1e-9;
    let opposite = |p: f64, q: f64| (p > eps && q < -eps) || (p < -eps && q > eps);
    opposite(cross(t.a, t.b, s.a), cross(t.a, t.b, s.b))
        && opposite(cross(s.a, s.b, t.a), cross(s.a, s.b, t.b))
}

fn bucket(v: f64) -> i64 {
    (v / BUCKET).floor() as i64
}

/// The borders (by index) whose curves cross another border's, or their own.
pub fn crossing(borders: &[Vec<CubicBezier>]) -> BTreeSet<usize> {
    let mut segments = Vec::new();
    for (border, curves) in borders.iter().enumerate() {
        let points = sample(curves);
        segments.extend(points.windows(2).map(|w| Segment {
            border,
            a: w[0],
            b: w[1],
        }));
    }
    let mut buckets: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, s) in segments.iter().enumerate() {
        let (x0, x1) = (bucket(s.a.0.min(s.b.0)), bucket(s.a.0.max(s.b.0)));
        let (y0, y1) = (bucket(s.a.1.min(s.b.1)), bucket(s.a.1.max(s.b.1)));
        for x in x0..=x1 {
            for y in y0..=y1 {
                buckets.entry((x, y)).or_default().push(i);
            }
        }
    }
    let mut found = BTreeSet::new();
    for members in buckets.values() {
        for (n, &i) in members.iter().enumerate() {
            for &j in &members[n + 1..] {
                let (s, t) = (&segments[i], &segments[j]);
                if segments_cross(s, t) {
                    found.insert(s.border);
                    found.insert(t.border);
                }
            }
        }
    }
    found
}
