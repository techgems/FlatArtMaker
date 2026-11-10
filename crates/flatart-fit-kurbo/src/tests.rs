use crate::KurboFitter;
use crate::deviation;
use flatart_domain::{CubicBezier, PxPoint};
use flatart_ports::CurveFitter;
use flatart_ports::testkit::{curve_fitter, fixtures};

#[test]
fn conforms() {
    curve_fitter::run(&KurboFitter);
}

fn p(x: f64, y: f64) -> PxPoint {
    PxPoint::new(x, y)
}

/// A closed 4 × 4 square, one point per pixel edge, starting at (0, 0).
fn square() -> Vec<PxPoint> {
    let mut points = Vec::new();
    points.extend((0..4).map(|i| p(f64::from(i), 0.0)));
    points.extend((0..4).map(|i| p(4.0, f64::from(i))));
    points.extend((0..4).map(|i| p(4.0 - f64::from(i), 4.0)));
    points.extend((0..=4).map(|i| p(0.0, 4.0 - f64::from(i))));
    points
}

#[test]
fn a_square_keeps_its_corners_sharp() {
    let curves = KurboFitter
        .fit(&square(), &fixtures::fit_settings())
        .unwrap();
    let expected: Vec<CubicBezier> = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (0.0, 0.0)]
        .windows(2)
        .map(|w| CubicBezier::line(p(w[0].0, w[0].1), p(w[1].0, w[1].1)))
        .collect();
    assert_eq!(
        curves, expected,
        "one straight curve per side, corner to corner"
    );
}

/// The angle between the tangents where `a` ends and `b` starts, in degrees.
fn kink(a: &CubicBezier, b: &CubicBezier) -> f64 {
    let (ux, uy) = (a.to.x.0 - a.ctrl2.x.0, a.to.y.0 - a.ctrl2.y.0);
    let (vx, vy) = (b.ctrl1.x.0 - b.from.x.0, b.ctrl1.y.0 - b.from.y.0);
    (ux * vy - uy * vx)
        .abs()
        .atan2(ux * vx + uy * vy)
        .to_degrees()
}

#[test]
fn a_threshold_above_the_turn_rounds_the_corners() {
    let settings = fixtures::fit_settings_with(0.5, 120.0);
    let points = square();
    let curves = KurboFitter.fit(&points, &settings).unwrap();
    for (i, pair) in curves.windows(2).enumerate() {
        let k = kink(&pair[0], &pair[1]);
        assert!(
            k < 1.0,
            "90° turns under a 120° threshold aren't corners, but curves {i} and {} meet at {k}°",
            i + 1
        );
    }
    assert!(deviation::within(&curves, &points, 0.5));
}

#[test]
fn an_arc_needs_far_fewer_curves_than_points() {
    let arc: Vec<_> = (0..=40)
        .map(|i| {
            let a = f64::from(i) / 40.0 * std::f64::consts::PI;
            p(20.0 * a.cos(), 20.0 * a.sin())
        })
        .collect();
    let curves = KurboFitter.fit(&arc, &fixtures::fit_settings()).unwrap();
    assert!(
        curves.len() <= 4,
        "{} curves for a half circle",
        curves.len()
    );
}

#[test]
fn a_pixel_rounded_circle_comes_out_smooth() {
    // Points on a circle snapped to the half-pixel grid, closed.
    let mut points: Vec<PxPoint> = (0..96)
        .map(|i| {
            let a = f64::from(i) / 96.0 * std::f64::consts::TAU;
            let snap = |v: f64| (v * 2.0).round() / 2.0;
            p(snap(30.0 * a.cos()), snap(30.0 * a.sin()))
        })
        .collect();
    points.dedup();
    points.push(points[0]);
    let curves = KurboFitter.fit(&points, &fixtures::fit_settings()).unwrap();
    assert!(curves.len() <= 12, "{} curves for a circle", curves.len());
    for (i, pair) in curves.windows(2).enumerate() {
        let k = kink(&pair[0], &pair[1]);
        assert!(k < 1.0, "curves {i} and {} meet at {k}°", i + 1);
    }
    assert!(
        kink(&curves[curves.len() - 1], &curves[0]) < 1.0,
        "kink where it closes"
    );
}

#[test]
fn a_loop_keeps_a_sharp_corner_where_it_starts() {
    // A teardrop: two tangents meeting at 90° at (0, 0), round the far side
    // of a circle.
    let (centre, radius) = (10.0 * std::f64::consts::SQRT_2, 10.0);
    let mut points = vec![p(0.0, 0.0)];
    points.extend((-27..=27).map(|i| {
        let a = f64::from(i) * 5.0_f64.to_radians();
        p(centre + radius * a.cos(), radius * a.sin())
    }));
    points.push(p(0.0, 0.0));
    let curves = KurboFitter.fit(&points, &fixtures::fit_settings()).unwrap();
    let k = kink(&curves[curves.len() - 1], &curves[0]);
    assert!(k > 60.0, "the 90° tip was smoothed to {k}°");
}

#[test]
fn bad_input_is_an_error_not_a_panic() {
    let settings = fixtures::fit_settings();
    for (case, points) in [
        ("one point", vec![p(0.0, 0.0)]),
        (
            "repeated point",
            vec![p(0.0, 0.0), p(0.0, 0.0), p(1.0, 0.0)],
        ),
        ("not finite", vec![p(0.0, 0.0), p(f64::NAN, 1.0)]),
    ] {
        assert!(KurboFitter.fit(&points, &settings).is_err(), "{case}");
    }
}

/// A small deterministic generator for the random polylines below.
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

#[test]
fn random_polylines_stay_within_tolerance_and_joined() {
    let mut rng = Lcg(9);
    for case in 0..300 {
        let n = 2 + (rng.unit() * 40.0) as usize;
        let wiggle = rng.unit() * 2.0;
        let mut heading: f64 = 0.0;
        let mut at = p(0.0, 0.0);
        let mut points = vec![at];
        for _ in 1..n {
            heading += (rng.unit() - 0.5) * wiggle;
            let step = 0.5 + rng.unit() * 3.0;
            at = p(at.x.0 + step * heading.cos(), at.y.0 + step * heading.sin());
            points.push(at);
        }
        let tolerance = 0.1 + rng.unit() * 1.9;
        let corner = rng.unit() * 180.0;
        let settings = fixtures::fit_settings_with(tolerance, corner);
        let curves = KurboFitter.fit(&points, &settings).unwrap();
        assert_eq!(curves[0].from, points[0], "case {case}");
        assert_eq!(curves[curves.len() - 1].to, points[n - 1], "case {case}");
        assert!(
            curves.windows(2).all(|w| w[0].to == w[1].from),
            "case {case}: not joined"
        );
        assert!(
            deviation::within(&curves, &points, tolerance + 1e-9),
            "case {case}: out of tolerance"
        );
    }
}
