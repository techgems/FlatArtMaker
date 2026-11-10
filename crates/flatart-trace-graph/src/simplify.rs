//! The points to fit for each border.
//!
//! A border's vertices form a pixel staircase. Fitting the staircase itself
//! would keep every step, since each is a 90° turn, so the smooth points
//! replace each straight run by its midpoint. A turn between two long runs
//! (at least `corner_run` cells each) is a real corner, not a step, so its
//! vertex is kept for the fitter to keep sharp.

use crate::grid::Vertex;

pub type Point = (f64, f64);

fn point(v: Vertex) -> Point {
    (v.0 as f64, v.1 as f64)
}

fn midpoint(a: Vertex, b: Vertex) -> Point {
    ((a.0 + b.0) as f64 / 2.0, (a.1 + b.1) as f64 / 2.0)
}

/// The indices where the chain turns, plus its two ends.
fn turns(vertices: &[Vertex]) -> Vec<usize> {
    let n = vertices.len();
    let mut turns = vec![0];
    for i in 1..n - 1 {
        let (a, b, c) = (vertices[i - 1], vertices[i], vertices[i + 1]);
        if (b.0 - a.0, b.1 - a.1) != (c.0 - b.0, c.1 - b.1) {
            turns.push(i);
        }
    }
    turns.push(n - 1);
    turns
}

/// The pixel edges as they are: the ends and every turn.
pub fn edge_points(vertices: &[Vertex]) -> Vec<Point> {
    turns(vertices)
        .into_iter()
        .map(|i| point(vertices[i]))
        .collect()
}

/// The ends, the midpoint of every straight run, and the vertices between
/// two runs at least `corner_run` long. A closed chain (first vertex equals
/// last) that doesn't start at such a corner starts at its first run's
/// midpoint instead, so the loop has no staircase step in it.
pub fn smooth_points(vertices: &[Vertex], closed: bool, corner_run: i64) -> Vec<Point> {
    let t = turns(vertices);
    let runs = t.len() - 1;
    let len = |r: usize| (t[r + 1] - t[r]) as i64;
    let mid = |r: usize| midpoint(vertices[t[r]], vertices[t[r + 1]]);
    let corner = |a: usize, b: usize| len(a) >= corner_run && len(b) >= corner_run;

    // The midpoint of each run, with each interior corner before it.
    let mut inner = Vec::with_capacity(2 * runs);
    for r in 0..runs {
        if r > 0 && corner(r - 1, r) {
            inner.push(point(vertices[t[r]]));
        }
        inner.push(mid(r));
    }

    let first = point(vertices[0]);
    if closed && !corner(runs - 1, 0) {
        // Start and end at the first run's midpoint.
        let start = inner[0];
        inner.push(start);
        return inner;
    }
    let mut points = Vec::with_capacity(inner.len() + 2);
    points.push(first);
    points.extend(inner);
    points.push(point(vertices[vertices.len() - 1]));
    points
}
