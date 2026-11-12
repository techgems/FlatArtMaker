use crate::VisionCortexFitter;
use flatart_domain::{CubicBezier, PxPoint};
use flatart_fit_support::deviation;
use flatart_ports::CurveFitter;
use flatart_ports::testkit::{curve_fitter, fixtures};

#[test]
fn conforms() {
    curve_fitter::run(&VisionCortexFitter);
}

fn p(x: f64, y: f64) -> PxPoint {
    PxPoint::new(x, y)
}

#[test]
fn a_square_keeps_its_corners_sharp() {
    let mut square = Vec::new();
    square.extend((0..4).map(|i| p(f64::from(i), 0.0)));
    square.extend((0..4).map(|i| p(4.0, f64::from(i))));
    square.extend((0..4).map(|i| p(4.0 - f64::from(i), 4.0)));
    square.extend((0..=4).map(|i| p(0.0, 4.0 - f64::from(i))));
    let curves = VisionCortexFitter
        .fit(&square, &fixtures::fit_settings())
        .unwrap();
    let expected: Vec<CubicBezier> = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (0.0, 0.0)]
        .windows(2)
        .map(|w| CubicBezier::line(p(w[0].0, w[0].1), p(w[1].0, w[1].1)))
        .collect();
    assert_eq!(curves, expected);
}

#[test]
fn an_arc_needs_far_fewer_curves_than_points() {
    let arc: Vec<_> = (0..=40)
        .map(|i| {
            let a = f64::from(i) / 40.0 * std::f64::consts::PI;
            p(20.0 * a.cos(), 20.0 * a.sin())
        })
        .collect();
    let curves = VisionCortexFitter
        .fit(&arc, &fixtures::fit_settings())
        .unwrap();
    assert!(
        curves.len() <= 6,
        "{} curves for a half circle",
        curves.len()
    );
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
        assert!(
            VisionCortexFitter.fit(&points, &settings).is_err(),
            "{case}"
        );
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

/// Checks everything the contract asks of `curves` fitted to `points`.
fn check(case: usize, curves: &[CubicBezier], points: &[PxPoint], tolerance: f64, corner: f64) {
    let settings = fixtures::fit_settings_with(tolerance, corner);
    assert_eq!(curves[0].from, points[0], "case {case}");
    assert_eq!(
        curves[curves.len() - 1].to,
        points[points.len() - 1],
        "case {case}"
    );
    assert!(
        curves.windows(2).all(|w| w[0].to == w[1].from),
        "case {case}: not joined"
    );
    assert!(
        deviation::within(curves, points, tolerance + 1e-9),
        "case {case}: out of tolerance"
    );
    curve_fitter::check_no_doubling_back(&format!("case {case}"), curves, points, &settings);
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
        let curves = VisionCortexFitter.fit(&points, &settings).unwrap();
        check(case, &curves, &points, tolerance, corner);
    }
}

/// Like the tracer's input: the midpoints of a random pixel staircase, so
/// there are many exactly straight stretches.
#[test]
fn random_staircases_finish_within_tolerance() {
    let mut rng = Lcg(11);
    for case in 0..300 {
        let (sx, sy) = (
            if rng.unit() < 0.5 { 1.0 } else { -1.0 },
            if rng.unit() < 0.5 { 1.0 } else { -1.0 },
        );
        let mut at = (0.0, 0.0);
        let mut points = vec![p(0.0, 0.0)];
        for r in 0..(4 + (rng.unit() * 40.0) as usize) {
            let len = 1.0 + (rng.unit() * 5.0).floor();
            let step = if r % 2 == 0 {
                (sx * len, 0.0)
            } else {
                (0.0, sy * len)
            };
            let mid = (at.0 + step.0 / 2.0, at.1 + step.1 / 2.0);
            at = (at.0 + step.0, at.1 + step.1);
            points.push(p(mid.0, mid.1));
        }
        points.push(p(at.0, at.1));
        let settings = fixtures::fit_settings();
        let curves = VisionCortexFitter.fit(&points, &settings).unwrap();
        let (tolerance, corner) = (settings.tolerance.get().0, settings.corner.get().0);
        check(case, &curves, &points, tolerance, corner);
    }
}

/// A closed loop with no corner: fitted as two halves, within tolerance.
#[test]
fn a_pixel_rounded_circle_fits_within_tolerance() {
    let mut points: Vec<PxPoint> = (0..96)
        .map(|i| {
            let a = f64::from(i) / 96.0 * std::f64::consts::TAU;
            let snap = |v: f64| (v * 2.0).round() / 2.0;
            p(snap(30.0 * a.cos()), snap(30.0 * a.sin()))
        })
        .collect();
    points.dedup();
    points.push(points[0]);
    let settings = fixtures::fit_settings();
    let curves = VisionCortexFitter.fit(&points, &settings).unwrap();
    assert!(curves.len() <= 16, "{} curves for a circle", curves.len());
    let (tolerance, corner) = (settings.tolerance.get().0, settings.corner.get().0);
    check(0, &curves, &points, tolerance, corner);
}
