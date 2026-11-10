//! Conformance suite for [`Triangulator`].

use super::geometry::{P, inside, mm, signed_area};
use super::ok;
use crate::Triangulator;
use flatart_domain::{MmPoint, PolygonWithHoles};

fn ring(points: &[(f64, f64)]) -> Vec<MmPoint> {
    points.iter().map(|&(x, y)| MmPoint::new(x, y)).collect()
}

/// The polygons every triangulator must handle.
pub fn cases() -> Vec<(&'static str, PolygonWithHoles)> {
    let square = ring(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
    let clockwise_square: Vec<_> = square.iter().rev().copied().collect();
    vec![
        (
            "square",
            PolygonWithHoles {
                outer: square.clone(),
                holes: vec![],
            },
        ),
        (
            "concave L",
            PolygonWithHoles {
                outer: ring(&[
                    (0.0, 0.0),
                    (10.0, 0.0),
                    (10.0, 4.0),
                    (4.0, 4.0),
                    (4.0, 10.0),
                    (0.0, 10.0),
                ]),
                holes: vec![],
            },
        ),
        (
            "clockwise square with a hole",
            PolygonWithHoles {
                outer: clockwise_square,
                holes: vec![ring(&[(3.0, 3.0), (7.0, 3.0), (7.0, 7.0), (3.0, 7.0)])],
            },
        ),
        (
            "two holes",
            PolygonWithHoles {
                outer: ring(&[(0.0, 0.0), (20.0, 0.0), (20.0, 10.0), (0.0, 10.0)]),
                holes: vec![
                    ring(&[(2.0, 2.0), (2.0, 8.0), (8.0, 8.0), (8.0, 2.0)]),
                    ring(&[(12.0, 2.0), (18.0, 2.0), (15.0, 8.0)]),
                ],
            },
        ),
    ]
}

/// Runs every check on `triangulator`.
pub fn run(triangulator: &dyn Triangulator) {
    for (case, polygon) in cases() {
        let triangles = ok(triangulator.triangulate(&polygon), case);
        check(&polygon, &triangles, case);
        let again = ok(triangulator.triangulate(&polygon), case);
        assert_eq!(triangles, again, "{case}: not deterministic");
    }
}

/// Checks one triangulation of `polygon`.
pub fn check(polygon: &PolygonWithHoles, triangles: &[[u32; 3]], case: &str) {
    let vertices: Vec<P> = polygon.vertices().map(mm).collect();
    let outer: Vec<P> = polygon.outer.iter().copied().map(mm).collect();
    let holes: Vec<Vec<P>> = polygon
        .holes
        .iter()
        .map(|h| h.iter().copied().map(mm).collect())
        .collect();

    let mut total = 0.0;
    for (t, tri) in triangles.iter().enumerate() {
        let corners = tri.map(|i| {
            *vertices
                .get(i as usize)
                .unwrap_or_else(|| panic!("{case}: triangle {t} points past the vertices"))
        });
        let area = signed_area(&corners);
        assert!(
            area > 1e-9,
            "{case}: triangle {t} is degenerate or clockwise (area {area})"
        );
        total += area;
        let centroid = (
            (corners[0].0 + corners[1].0 + corners[2].0) / 3.0,
            (corners[0].1 + corners[1].1 + corners[2].1) / 3.0,
        );
        assert!(
            inside(centroid, &outer) && !holes.iter().any(|h| inside(centroid, h)),
            "{case}: triangle {t} lies outside the polygon or in a hole"
        );
    }
    let expected =
        signed_area(&outer).abs() - holes.iter().map(|h| signed_area(h).abs()).sum::<f64>();
    assert!(
        (total - expected).abs() <= 1e-9 * expected.max(1.0),
        "{case}: triangles cover {total} mm² but the polygon is {expected} mm²"
    );
}
