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
        curve_fitter::check_no_doubling_back(&format!("case {case}"), &curves, &points, &settings);
    }
}

/// From Jolteon: a traced dot whose sides are exactly straight. kurbo never
/// finished fitting those.
#[test]
fn a_loop_with_straight_stretches_finishes() {
    let raw = [
        (652.0, 549.0),
        (652.5, 549.25),
        (653.0, 549.5),
        (653.5, 549.75),
        (654.0, 550.0),
        (654.5, 550.25),
        (655.0, 550.5),
        (655.5, 551.0),
        (655.75, 551.5),
        (656.0, 552.5),
        (655.75, 553.5),
        (655.5, 554.0),
        (655.0, 554.5),
        (654.5, 555.0),
        (654.0, 555.5),
        (653.5, 555.75),
        (652.5, 556.0),
        (651.5, 555.75),
        (651.0, 555.5),
        (650.5, 555.0),
        (650.0, 554.5),
        (649.5, 554.0),
        (649.25, 553.5),
        (649.0, 553.0),
        (649.25, 552.5),
        (649.5, 552.0),
        (649.75, 551.5),
        (650.0, 551.0),
        (650.25, 550.5),
        (650.5, 550.0),
        (651.0, 549.5),
        (651.5, 549.25),
        (652.0, 549.0),
    ];
    let points: Vec<_> = raw.iter().map(|&(x, y)| p(x, y)).collect();
    let settings = fixtures::fit_settings();
    let curves = KurboFitter.fit(&points, &settings).unwrap();
    assert!(deviation::within(&curves, &points, 0.5));
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
        let curves = KurboFitter.fit(&points, &settings).unwrap();
        assert!(
            deviation::within(&curves, &points, 0.5 + 1e-9),
            "case {case}"
        );
        curve_fitter::check_no_doubling_back(&format!("case {case}"), &curves, &points, &settings);
    }
}

/// Runs `f` on its own thread and fails the test if it takes longer than
/// `seconds`, so a regression to a hang fails the test rather than stalling
/// the run.
fn finishes_within<T: Send + 'static>(seconds: u64, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(std::time::Duration::from_secs(seconds))
        .unwrap_or_else(|_| panic!("didn't finish in {seconds} s"))
}

/// A cubic kurbo's fitter proposed on the Zapdos image: one control point is
/// about 2e13 px away. Unpatched kurbo 0.13.1's `inv_arclen` never returned
/// on it (see vendor/kurbo/PATCHES.md).
#[test]
fn inv_arclen_finishes_on_a_very_long_cubic() {
    use kurbo::{CubicBez, ParamCurveArclen};
    let t = finishes_within(10, || {
        let c = CubicBez::new(
            (394.25, 340.5),
            (394.250_000_000_003_35, 340.499_999_999_989_65),
            (-5_422_853_629_292.885, 21_691_414_519_092.727),
            (397.0, 329.5),
        );
        c.inv_arclen(0.5 * c.arclen(1e-9), 1e-9)
    });
    assert!((0.0..=1.0).contains(&t), "{t}");
}

/// From Zapdos: one side of a lightning bolt, a smooth run with no corner.
/// Fitting it hung for good in unpatched kurbo 0.13.1.
#[test]
fn a_lightning_bolt_edge_finishes() {
    let raw = [
        (396.0, 348.0),
        (395.75, 348.5),
        (395.5, 348.75),
        (394.5, 349.0),
        (393.5, 348.75),
        (393.25, 348.5),
        (393.0, 346.5),
        (393.25, 344.5),
        (393.5, 344.0),
        (393.75, 343.5),
        (394.0, 342.0),
        (394.25, 340.5),
        (394.5, 340.0),
        (394.75, 339.5),
        (395.0, 338.0),
        (395.25, 336.5),
        (395.5, 336.0),
        (395.75, 335.5),
        (396.0, 333.5),
        (396.25, 331.5),
        (396.5, 331.0),
        (396.75, 330.5),
        (397.0, 329.5),
        (397.25, 328.5),
        (397.5, 328.0),
        (397.75, 327.5),
        (398.0, 326.5),
        (398.25, 325.5),
        (398.5, 325.0),
        (398.75, 324.5),
        (399.0, 323.0),
        (399.25, 321.5),
        (399.5, 321.0),
        (399.75, 320.5),
        (400.0, 319.5),
        (400.25, 318.5),
        (400.5, 318.0),
        (400.75, 317.5),
        (401.0, 316.5),
        (401.25, 315.5),
        (401.5, 315.0),
        (401.75, 314.5),
        (402.0, 313.0),
        (402.25, 311.5),
        (402.5, 311.0),
    ];
    let points: Vec<_> = raw.iter().map(|&(x, y)| p(x, y)).collect();
    let check = points.clone();
    let curves = finishes_within(10, move || {
        KurboFitter.fit(&points, &fixtures::fit_settings()).unwrap()
    });
    assert!(deviation::within(&curves, &check, 0.5));
}
