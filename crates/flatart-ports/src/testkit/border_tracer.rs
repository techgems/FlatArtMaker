//! Conformance suite for [`BorderTracer`].

use super::geometry::{sample, segments_cross};
use super::{fixtures, ok};
use crate::{BorderTracer, CurveFitter};
use flatart_domain::{BorderEnds, BorderMap, Label, LabelMap, Loop, PxPoint};
use std::collections::HashMap;

/// Runs every check on `tracer`, fitting curves with `fitter`.
pub fn run(tracer: &dyn BorderTracer, fitter: &dyn CurveFitter) {
    let settings = fixtures::trace_settings();
    for (case, labels) in fixtures::all_label_maps() {
        let map = ok(tracer.trace(&labels, &settings, fitter), case);
        check(&labels, &map, case);
        let again = ok(tracer.trace(&labels, &settings, fitter), case);
        assert_eq!(map, again, "{case}: not deterministic");
    }
}

/// Checks one border map against the label map it was traced from.
pub fn check(labels: &LabelMap, map: &BorderMap, case: &str) {
    let k = f64::from(labels.upscale().get());
    assert_eq!(
        (map.width.0, map.height.0),
        (
            f64::from(labels.width()) / k,
            f64::from(labels.height()) / k
        ),
        "{case}: size must be the label map's size in source pixels"
    );
    check_borders(map, case);
    check_faces(map, case);
    check_sharing(map, case);
    check_crossings(map, case);

    let expected = count_areas(labels);
    assert_eq!(
        map.faces.len(),
        expected,
        "{case}: {} faces for {expected} 4-connected areas",
        map.faces.len()
    );
}

fn check_borders(map: &BorderMap, case: &str) {
    for (i, border) in map.borders.iter().enumerate() {
        let (Some(start), Some(end)) = (border.start(), border.end()) else {
            panic!("{case}: border {i} has no curve");
        };
        assert_ne!(
            border.left, border.right,
            "{case}: border {i} has the same label on both sides"
        );
        for (j, pair) in border.curve.windows(2).enumerate() {
            assert_eq!(
                pair[0].to,
                pair[1].from,
                "{case}: border {i} breaks between curves {j} and {}",
                j + 1
            );
        }
        match border.ends {
            BorderEnds::Closed => {
                assert_eq!(start, end, "{case}: closed border {i} doesn't close");
            }
            BorderEnds::Open { start: s, end: e } => {
                let junction = |id: flatart_domain::JunctionId| {
                    map.junction(id)
                        .unwrap_or_else(|| {
                            panic!("{case}: border {i} names missing junction {}", id.0)
                        })
                        .at
                };
                assert_eq!(
                    start,
                    junction(s),
                    "{case}: border {i} doesn't start at its junction"
                );
                assert_eq!(
                    end,
                    junction(e),
                    "{case}: border {i} doesn't end at its junction"
                );
            }
        }
    }
}

/// The start and end of a loop step, in walking order.
fn step_ends(map: &BorderMap, step: &flatart_domain::LoopStep, case: &str) -> (PxPoint, PxPoint) {
    let border = map
        .border(step.border)
        .unwrap_or_else(|| panic!("{case}: a loop names missing border {}", step.border.0));
    let (start, end) = (border.start(), border.end());
    let (Some(start), Some(end)) = (start, end) else {
        panic!("{case}: border {} has no curve", step.border.0);
    };
    if step.reversed {
        (end, start)
    } else {
        (start, end)
    }
}

fn check_loop(map: &BorderMap, lp: &Loop, case: &str, what: &str) {
    assert!(!lp.steps.is_empty(), "{case}: {what} is empty");
    let n = lp.steps.len();
    for i in 0..n {
        let (_, end) = step_ends(map, &lp.steps[i], case);
        let (next_start, _) = step_ends(map, &lp.steps[(i + 1) % n], case);
        assert_eq!(end, next_start, "{case}: {what} breaks after step {i}");
    }
}

fn check_faces(map: &BorderMap, case: &str) {
    for (f, face) in map.faces.iter().enumerate() {
        assert!(
            !Label::from(face.colour).is_outside(),
            "{case}: face {f} is outside"
        );
        check_loop(map, &face.outer, case, &format!("face {f} outer loop"));
        for (h, hole) in face.holes.iter().enumerate() {
            check_loop(map, hole, case, &format!("face {f} hole {h}"));
        }
    }
}

fn check_sharing(map: &BorderMap, case: &str) {
    // (border, label) → number of loops of faces with that label using the border.
    let mut uses: HashMap<(u32, Label), usize> = HashMap::new();
    for face in &map.faces {
        let label = Label::from(face.colour);
        for lp in std::iter::once(&face.outer).chain(&face.holes) {
            for step in &lp.steps {
                *uses.entry((step.border.0, label)).or_default() += 1;
            }
        }
    }
    for (i, border) in map.borders.iter().enumerate() {
        for side in [border.left, border.right] {
            if !side.is_outside() {
                let n = uses.remove(&(i as u32, side)).unwrap_or(0);
                assert_eq!(
                    n, 1,
                    "{case}: border {i} is used {n} times by label {} (expected once)",
                    side.0
                );
            }
        }
    }
    if let Some(((border, label), _)) = uses.into_iter().next() {
        panic!(
            "{case}: a face of label {} uses border {border}, which doesn't touch it",
            label.0
        );
    }
}

fn check_crossings(map: &BorderMap, case: &str) {
    let lines: Vec<_> = map.borders.iter().map(|b| sample(&b.curve, 16)).collect();
    for (i, a) in lines.iter().enumerate() {
        for (j, b) in lines.iter().enumerate().skip(i + 1) {
            for sa in a.windows(2) {
                for sb in b.windows(2) {
                    assert!(
                        !segments_cross(sa[0], sa[1], sb[0], sb[1]),
                        "{case}: borders {i} and {j} cross near {:?}",
                        sa[0]
                    );
                }
            }
        }
    }
}

/// Counts 4-connected areas of non-outside labels.
fn count_areas(labels: &LabelMap) -> usize {
    let (w, h) = (labels.width(), labels.height());
    let mut seen = vec![false; (w * h) as usize];
    let mut areas = 0;
    for start in 0..(w * h) {
        let label = labels.labels()[start as usize];
        if seen[start as usize] || label.is_outside() {
            continue;
        }
        areas += 1;
        let mut stack = vec![start];
        seen[start as usize] = true;
        while let Some(cell) = stack.pop() {
            let (x, y) = (cell % w, cell / w);
            let neighbours = [
                (x > 0).then(|| cell - 1),
                (x + 1 < w).then(|| cell + 1),
                (y > 0).then(|| cell - w),
                (y + 1 < h).then(|| cell + w),
            ];
            for n in neighbours.into_iter().flatten() {
                if !seen[n as usize] && labels.labels()[n as usize] == label {
                    seen[n as usize] = true;
                    stack.push(n);
                }
            }
        }
    }
    areas
}
