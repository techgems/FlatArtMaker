//! Conformance suite for [`CurveFitter`].

use super::geometry::{P, dist_to_polyline, dot, end_direction, px, sample, start_direction, turn};
use super::{fixtures, ok};
use crate::CurveFitter;
use flatart_domain::settings::FitSettings;
use flatart_domain::{CubicBezier, PxPoint};

/// Runs every check on `fitter`.
pub fn run(fitter: &dyn CurveFitter) {
    let p = PxPoint::new;
    let straight: Vec<_> = (0..10).map(|i| p(f64::from(i), 0.0)).collect();
    let corner = [
        p(0.0, 0.0),
        p(1.0, 0.0),
        p(2.0, 0.0),
        p(3.0, 0.0),
        p(3.0, 1.0),
        p(3.0, 2.0),
        p(3.0, 3.0),
    ];
    let arc: Vec<_> = (0..=20)
        .map(|i| {
            let a = f64::from(i) / 20.0 * std::f64::consts::FRAC_PI_2;
            p(10.0 * a.cos(), 10.0 * a.sin())
        })
        .collect();
    let two = [p(0.0, 0.0), p(5.0, 5.0)];

    let settings = fixtures::fit_settings();
    let tight = fixtures::fit_settings_with(0.1, 60.0);
    for (case, points, settings) in [
        ("straight", &straight[..], settings),
        ("corner", &corner[..], settings),
        ("arc", &arc[..], settings),
        ("arc, tight", &arc[..], tight),
        ("two points", &two[..], settings),
    ] {
        check(fitter, case, points, &settings);
    }

    let curves = ok(fitter.fit(&straight, &settings), "straight");
    for (i, c) in curves.iter().enumerate() {
        for q in [c.from, c.ctrl1, c.ctrl2, c.to] {
            assert!(
                q.y.0.abs() < 1e-9,
                "straight: curve {i} leaves the line (control point y = {})",
                q.y.0
            );
        }
    }

    let curves = ok(fitter.fit(&corner, &settings), "corner");
    assert!(
        curves.iter().any(|c| c.to == corner[3]),
        "corner: a 90° turn with a 60° threshold must end a curve exactly at (3, 0)"
    );
}

fn check(fitter: &dyn CurveFitter, case: &str, points: &[PxPoint], settings: &FitSettings) {
    let curves = ok(fitter.fit(points, settings), case);
    assert!(!curves.is_empty(), "{case}: no curves");
    assert_eq!(
        curves[0].from, points[0],
        "{case}: first curve must start exactly at the first point"
    );
    assert_eq!(
        curves[curves.len() - 1].to,
        points[points.len() - 1],
        "{case}: last curve must end exactly at the last point"
    );
    for (i, pair) in curves.windows(2).enumerate() {
        assert_eq!(
            pair[0].to,
            pair[1].from,
            "{case}: curve {} doesn't start where curve {i} ends",
            i + 1
        );
    }

    let tolerance = settings.tolerance.get().0 + 1e-6;
    let input: Vec<P> = points.iter().map(|&q| px(q)).collect();
    let fitted = sample(&curves, 64);
    for (i, &q) in input.iter().enumerate() {
        let d = dist_to_polyline(q, &fitted);
        assert!(
            d <= tolerance,
            "{case}: input point {i} is {d} px from the curves (tolerance {tolerance})"
        );
    }
    for &q in &fitted {
        let d = dist_to_polyline(q, &input);
        assert!(
            d <= tolerance,
            "{case}: the curves stray {d} px from the polyline at {q:?} (tolerance {tolerance})"
        );
    }

    check_no_doubling_back(case, &curves, points, settings);

    let again: Vec<CubicBezier> = ok(fitter.fit(points, settings), case);
    assert_eq!(curves, again, "{case}: not deterministic");
}

/// Checks the contract's "no doubling back": `curves`, fitted to `points`,
/// leave and reach the polyline's ends along it, and only turn back where
/// two curves meet at a corner or where the polyline itself turns back
/// (more than 90°). Adapters can run it on their own cases.
pub fn check_no_doubling_back(
    case: &str,
    curves: &[CubicBezier],
    points: &[PxPoint],
    settings: &FitSettings,
) {
    let corner = settings.corner.get().0;
    let input: Vec<P> = points.iter().map(|&q| px(q)).collect();
    let n = input.len();
    for (i, c) in curves.iter().enumerate() {
        let (a, b) = (px(c.from), px(c.to));
        let chord = (b.0 - a.0, b.1 - a.1);
        let length2 = dot(chord, chord);
        let slack = 1e-9 * length2;
        for k in 0..=1024 {
            let q = px(c.point_at(f64::from(k) / 1024.0));
            let along = dot((q.0 - a.0, q.1 - a.1), chord);
            assert!(
                along >= -slack && along <= length2 + slack,
                "{case}: curve {i} runs back behind its start or past its end at {q:?}"
            );
        }
    }
    let along = |a: P, b: P| (b.0 - a.0, b.1 - a.1);
    assert!(
        dot(start_direction(&curves[0]), along(input[0], input[1])) > 0.0,
        "{case}: the first curve sets off against the polyline"
    );
    assert!(
        dot(
            end_direction(&curves[curves.len() - 1]),
            along(input[n - 2], input[n - 1])
        ) > 0.0,
        "{case}: the last curve arrives against the polyline"
    );
    for (i, pair) in curves.windows(2).enumerate() {
        if dot(end_direction(&pair[0]), start_direction(&pair[1])) > 0.0 {
            continue;
        }
        let at = px(pair[0].to);
        // A corner, or a point where the polyline itself turns back.
        let allowed = corner.min(90.0);
        let at_corner =
            (1..n - 1).any(|k| input[k] == at && turn(input[k - 1], at, input[k + 1]) > allowed);
        assert!(
            at_corner,
            "{case}: curve {} turns back over curve {i} at {at:?}, where the polyline doesn't",
            i + 1
        );
    }
}
