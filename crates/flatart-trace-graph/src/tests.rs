use crate::GraphTracer;
use flatart_domain::settings::TraceUpscale;
use flatart_domain::{BorderEnds, BorderMap, Label, LabelMap, PxPoint};
use flatart_fit_kurbo::KurboFitter;
use flatart_ports::BorderTracer;
use flatart_ports::testkit::{border_tracer, fixtures};
use proptest::prelude::*;

#[test]
fn conforms() {
    border_tracer::run(&GraphTracer, &KurboFitter);
}

fn trace(map: &LabelMap) -> BorderMap {
    let traced = GraphTracer
        .trace(map, &fixtures::trace_settings(), &KurboFitter)
        .unwrap();
    border_tracer::check(map, &traced, "trace");
    traced
}

fn p(x: f64, y: f64) -> PxPoint {
    PxPoint::new(x, y)
}

#[test]
fn an_island_gives_two_closed_borders_and_a_hole() {
    let traced = trace(&fixtures::nested_labels());
    assert!(traced.junctions.is_empty());
    assert_eq!(traced.borders.len(), 2);
    assert!(traced.borders.iter().all(|b| b.ends == BorderEnds::Closed));
    assert_eq!(traced.faces.len(), 2);
    assert_eq!(traced.faces[0].holes.len(), 1);
    assert!(traced.faces[1].holes.is_empty());
}

#[test]
fn three_labels_meet_at_a_junction() {
    let traced = trace(&fixtures::three_labels());
    assert!(traced.junctions.iter().any(|j| j.at == p(2.0, 2.0)));
    // Each pair meets once.
    let pairs: Vec<_> = traced
        .borders
        .iter()
        .filter(|b| !b.left.is_outside() && !b.right.is_outside())
        .collect();
    assert_eq!(pairs.len(), 3, "{pairs:?}");
}

#[test]
fn diagonal_cells_stay_separate_faces() {
    let traced = trace(&fixtures::diagonal_labels());
    assert_eq!(traced.faces.len(), 4);
    assert!(traced.junctions.iter().any(|j| j.at == p(1.0, 1.0)));
}

#[test]
fn a_square_keeps_its_corners() {
    let mut rows = vec!["........"];
    rows.extend([".000000."; 6]);
    rows.push("........");
    let traced = trace(&fixtures::labels(&rows, 1));
    let ends: Vec<PxPoint> = traced.borders[0].curve.iter().map(|c| c.to).collect();
    for corner in [p(7.0, 1.0), p(7.0, 7.0), p(1.0, 7.0)] {
        assert!(ends.contains(&corner), "{corner:?} isn't a curve end");
    }
    assert_eq!(traced.borders[0].start(), Some(p(1.0, 1.0)));
}

#[test]
fn a_staircase_becomes_a_smooth_line() {
    // Label 1 above the diagonal, 0 below: a 1-pixel staircase.
    let rows: Vec<String> = (0..12)
        .map(|y| (0..12).map(|x| if x > y { '1' } else { '0' }).collect())
        .collect();
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let traced = trace(&fixtures::labels(&rows, 1));
    let diagonal = traced
        .borders
        .iter()
        .find(|b| !b.left.is_outside() && !b.right.is_outside())
        .unwrap();
    // One straight line, plus at most a short curve at each end where the
    // staircase leaves its junction.
    assert!(
        diagonal.curve.len() <= 3,
        "{} curves: the staircase wasn't smoothed",
        diagonal.curve.len()
    );
}

#[test]
fn upscaled_maps_trace_in_source_pixels() {
    let traced = trace(&fixtures::upscaled(&fixtures::nested_labels(), 2));
    assert_eq!((traced.width.0, traced.height.0), (6.0, 6.0));
    let outer = &traced.borders[0];
    assert_eq!(outer.start(), Some(p(1.0, 1.0)));
}

#[test]
fn a_map_with_nothing_inside_has_no_faces() {
    let traced = trace(&fixtures::labels(&["...", "..."], 1));
    assert!(traced.borders.is_empty() && traced.faces.is_empty());
}

fn label_map() -> impl Strategy<Value = LabelMap> {
    (1u32..10, 1u32..10, 1u8..=2).prop_flat_map(|(w, h, k)| {
        // Few labels, so areas are big enough to have holes and junctions.
        let cell = prop_oneof![
            3 => (0u16..3).prop_map(Label),
            1 => Just(Label::OUTSIDE),
        ];
        proptest::collection::vec(cell, (w * h) as usize).prop_map(move |cells| {
            let map = LabelMap::new(w, h, TraceUpscale::new(1).unwrap(), cells).unwrap();
            fixtures::upscaled(&map, k)
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn random_maps_keep_the_invariants(map in label_map(), tolerance in 0.1f64..2.0, corner in 0f64..180.0) {
        let settings = flatart_domain::settings::TraceSettings {
            fit: fixtures::fit_settings_with(tolerance, corner),
        };
        let traced = GraphTracer.trace(&map, &settings, &KurboFitter).unwrap();
        border_tracer::check(&map, &traced, "random");
        let again = GraphTracer.trace(&map, &settings, &KurboFitter).unwrap();
        prop_assert_eq!(traced, again);
    }
}

/// A synthetic map the size of Jolteon (451 × 437) at the default ×2
/// upscale: overlapping ellipses of 8 colours inside a round outline, with
/// thin stripes and small dots.
fn jolteon_sized() -> LabelMap {
    let (w, h) = (451i32, 437i32);
    let shapes: [(f64, f64, f64, f64, u16); 8] = [
        (225.0, 140.0, 90.0, 60.0, 1),
        (150.0, 260.0, 70.0, 110.0, 2),
        (300.0, 270.0, 80.0, 70.0, 3),
        (225.0, 330.0, 120.0, 40.0, 4),
        (110.0, 120.0, 40.0, 70.0, 5),
        (340.0, 120.0, 40.0, 70.0, 6),
        (225.0, 220.0, 30.0, 30.0, 7),
        (200.0, 150.0, 12.0, 9.0, 0),
    ];
    let mut cells = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let (dx, dy) = (fx - 225.5, fy - 218.5);
            let label = if dx * dx + dy * dy > 210.0 * 210.0 {
                Label::OUTSIDE
            } else if (x + 2 * y) % 37 < 2 && fy > 300.0 {
                Label(5) // thin diagonal stripes
            } else if (x % 23 == 0) && (y % 19 == 0) {
                Label(6) // single-pixel dots
            } else {
                shapes
                    .iter()
                    .find(|&&(cx, cy, rx, ry, _)| {
                        let (u, v) = ((fx - cx) / rx, (fy - cy) / ry);
                        u * u + v * v <= 1.0
                    })
                    .map_or(Label(0), |s| Label(s.4))
            };
            cells.push(label);
        }
    }
    let map = LabelMap::new(w as u32, h as u32, TraceUpscale::new(1).unwrap(), cells).unwrap();
    fixtures::upscaled(&map, 2)
}

/// Run with `cargo test --release -p flatart-trace-graph -- --ignored`.
#[test]
#[ignore = "timing: meaningful only in release"]
fn traces_a_jolteon_sized_map_in_under_a_second() {
    let map = jolteon_sized();
    let start = std::time::Instant::now();
    let traced = GraphTracer
        .trace(&map, &fixtures::trace_settings(), &KurboFitter)
        .unwrap();
    let elapsed = start.elapsed();
    let curves: usize = traced.borders.iter().map(|b| b.curve.len()).sum();
    println!(
        "{} junctions, {} borders, {curves} curves, {} faces in {elapsed:?}",
        traced.junctions.len(),
        traced.borders.len(),
        traced.faces.len()
    );
    assert!(elapsed.as_secs_f64() < 1.0, "took {elapsed:?}");
    border_tracer::check(&map, &traced, "jolteon-sized");
}
