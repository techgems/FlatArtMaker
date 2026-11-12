use crate::ITriangulator;
use flatart_ports::testkit::triangulator;

#[test]
fn conforms() {
    triangulator::run(&ITriangulator);
}

mod simple {
    use crate::ITriangulator;
    use flatart_domain::{Error, MmPoint, PolygonWithHoles};
    use flatart_ports::Triangulator;

    fn ring(points: &[(f64, f64)]) -> Vec<MmPoint> {
        points.iter().map(|&(x, y)| MmPoint::new(x, y)).collect()
    }

    fn rejected(polygon: &PolygonWithHoles) {
        match ITriangulator.triangulate(polygon) {
            Err(Error::InvalidInput(message)) => assert!(message.contains("simple"), "{message}"),
            other => panic!("expected an invalid-input error, got {other:?}"),
        }
    }

    /// From a traced image: a curve doubled back on its neighbour, and the
    /// flattened ring crossed itself. i_triangle crashed on it.
    #[test]
    fn a_ring_that_crosses_itself_is_an_error() {
        rejected(&PolygonWithHoles {
            outer: ring(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (2.0, -1.0), (0.0, 4.0)]),
            holes: vec![],
        });
    }

    #[test]
    fn a_hole_crossing_the_outer_ring_is_an_error() {
        rejected(&PolygonWithHoles {
            outer: ring(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]),
            holes: vec![ring(&[(1.0, 1.0), (5.0, 1.0), (1.0, 3.0)])],
        });
    }

    #[test]
    fn a_point_on_another_edge_is_an_error() {
        rejected(&PolygonWithHoles {
            outer: ring(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]),
            holes: vec![ring(&[(2.0, 0.0), (3.0, 1.0), (1.0, 1.0)])],
        });
    }

    #[test]
    fn a_ring_that_folds_back_on_itself_is_an_error() {
        rejected(&PolygonWithHoles {
            outer: ring(&[(0.0, 0.0), (4.0, 0.0), (2.0, 0.0), (2.0, 3.0)]),
            holes: vec![],
        });
    }

    /// The contract allows rings to touch at a point both list.
    #[test]
    fn a_hole_touching_the_outer_ring_at_a_shared_point_is_fine() {
        let polygon = PolygonWithHoles {
            outer: ring(&[(0.0, 0.0), (2.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]),
            holes: vec![ring(&[(2.0, 0.0), (3.0, 2.0), (1.0, 2.0)])],
        };
        assert!(ITriangulator.triangulate(&polygon).is_ok());
    }
}
