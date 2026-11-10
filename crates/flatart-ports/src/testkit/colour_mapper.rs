//! Conformance suite for [`ColourMapper`].

use super::{fixtures, ok};
use crate::ColourMapper;
use flatart_domain::{
    Detection, DropReason, EditLog, EditOp, GroupId, Label, MAX_OUTPUT_COLOURS, MappedColours,
    Oklab, OutputColourId, Paint, PxPoint, Rgb,
};

fn log(ops: Vec<EditOp>) -> EditLog {
    let mut log = EditLog::new();
    ops.into_iter().for_each(|op| log.push(op));
    log
}

fn target_of(mapped: &MappedColours, label: u16) -> OutputColourId {
    mapped
        .mapping
        .target(Label(label))
        .unwrap_or_else(|| panic!("detected label {label} has no target"))
}

fn report(mapped: &MappedColours) -> Vec<(usize, DropReason)> {
    mapped
        .dropped_edits
        .iter()
        .map(|d| (d.position, d.reason))
        .collect()
}

/// Runs every check on `mapper`, using [`fixtures::detection`].
pub fn run(mapper: &dyn ColourMapper) {
    let detection = fixtures::detection();
    let map = |edits: &EditLog, target: u8, case: &str| {
        let settings = fixtures::mapping_settings(target);
        let mapped = ok(mapper.map(&detection, edits, &settings), case);
        check(&detection, edits, &mapped, case);
        mapped
    };
    let none = EditLog::new();
    let colour = |label: usize| detection.palette.colours()[label].colour;

    automatic_pass(&map);

    // The rest asks for 3 colours: 0 = black + near-black, 1 = red, 2 = white.
    let auto = map(&none, 3, "automatic");
    let red = target_of(&auto, 3);

    // Map, including a stored colour that drifted slightly (stable IDs).
    let drifted = Oklab::new(colour(2).l() - 0.005, 0.0, 0.0).expect("finite");
    for (stored, case) in [(colour(2), "map"), (drifted, "map after drift")] {
        let edits = log(vec![EditOp::Map {
            detected: stored,
            output: red,
        }]);
        let m = map(&edits, 3, case);
        assert_eq!(
            target_of(&m, 2),
            red,
            "{case}: white must now map to red's output"
        );
        assert!(
            m.dropped_edits.is_empty(),
            "{case}: nothing should be dropped"
        );
    }

    // Dropped operations are reported, and the rest still apply.
    let far = Oklab::new(0.5, -0.2, -0.2).expect("finite");
    let edits = log(vec![
        EditOp::Map {
            detected: colour(2),
            output: OutputColourId(99),
        },
        EditOp::Map {
            detected: far,
            output: red,
        },
        EditOp::Map {
            detected: colour(2),
            output: red,
        },
    ]);
    let m = map(&edits, 3, "dropped");
    assert_eq!(
        report(&m),
        [
            (0, DropReason::UnknownOutputColour),
            (1, DropReason::NoMatchingDetectedColour)
        ],
        "dropped: wrong report"
    );
    assert_eq!(
        target_of(&m, 2),
        red,
        "dropped: the valid operation must still apply"
    );

    // Undo: an undone operation has no effect.
    let mut undone = log(vec![EditOp::Map {
        detected: colour(2),
        output: red,
    }]);
    undone.undo();
    assert_eq!(
        map(&undone, 3, "undo"),
        auto,
        "undo: must match the automatic result"
    );

    // Paint and delete: the paint follows its colour to the reassignment target.
    let black = target_of(&auto, 0);
    let paint = Paint {
        at: PxPoint::new(1.0, 1.0),
        colour: black,
    };
    let m = map(&log(vec![EditOp::PaintRegion(paint)]), 3, "paint");
    assert_eq!(m.paints, [paint], "paint: must pass the paint through");
    let edits = log(vec![
        EditOp::PaintRegion(paint),
        EditOp::DeleteOutput {
            id: black,
            reassign_to: red,
        },
    ]);
    let m = map(&edits, 3, "delete");
    assert!(
        m.palette.get(black).is_none(),
        "delete: the colour must be gone"
    );
    assert_eq!(
        target_of(&m, 0),
        red,
        "delete: its detected colours must move to the target"
    );
    assert_eq!(
        m.paints[0].colour, red,
        "delete: its paints must move to the target"
    );

    // Add up to the limit, then the next one is dropped.
    let first = OutputColourId::FIRST_ADDED.0;
    let adds = (0..14u16)
        .map(|i| EditOp::AddOutput {
            id: OutputColourId(first + i),
            display: Rgb {
                r: 15 * i as u8,
                g: 100,
                b: 200,
            },
        })
        .collect();
    let m = map(&log(adds), 3, "add");
    assert_eq!(
        m.palette.colours().len(),
        MAX_OUTPUT_COLOURS,
        "add: 3 automatic + 13 added"
    );
    assert_eq!(
        report(&m),
        [(13, DropReason::TooManyOutputColours)],
        "add: the 17th colour must be dropped"
    );

    // Set base.
    let edits = log(vec![EditOp::SetBase { id: Some(red) }]);
    assert_eq!(
        map(&edits, 3, "set base").palette.base(),
        Some(red),
        "set base: must use the chosen colour"
    );
}

/// The automatic pass merges the most similar colours down to the target,
/// following the similarity tree.
fn automatic_pass(map: &impl Fn(&EditLog, u8, &str) -> MappedColours) {
    let none = EditLog::new();
    let all = map(&none, 8, "target above the detected count");
    assert_eq!(
        all.palette.colours().len(),
        4,
        "target above the detected count: nothing merges"
    );

    let auto = map(&none, 3, "automatic");
    let ids: Vec<OutputColourId> = auto.palette.colours().iter().map(|c| c.id).collect();
    assert_eq!(
        ids,
        (0..3).map(OutputColourId).collect::<Vec<_>>(),
        "automatic: black and near-black (2 ΔE apart) must merge, leaving outputs 0, 1, 2"
    );
    assert_eq!(
        target_of(&auto, 0),
        target_of(&auto, 1),
        "automatic: black and near-black must share an output"
    );
    assert_eq!(
        auto.palette.base(),
        Some(target_of(&auto, 0)),
        "automatic: the base must default to the colour most common on the outer edge (black)"
    );
    assert_eq!(auto, map(&none, 3, "automatic"), "not deterministic");
    assert_eq!(
        auto.similarity.merges().first().map(|m| m.joined),
        Some([GroupId(0), GroupId(1)]),
        "similarity: black and near-black must be the first merge"
    );

    for target in 1..=4u8 {
        let mapped = map(&none, target, "similarity");
        let groups = mapped.similarity.groups(usize::from(target));
        for a in 0..4u16 {
            for b in a + 1..4 {
                let together = groups
                    .iter()
                    .any(|g| g.contains(&Label(a)) && g.contains(&Label(b)));
                assert_eq!(
                    target_of(&mapped, a) == target_of(&mapped, b),
                    together,
                    "similarity: with target {target}, labels {a} and {b} must share an \
                     output exactly when the tree groups them"
                );
            }
        }
    }
}

/// Checks the contract clauses every result must meet.
pub fn check(detection: &Detection, edits: &EditLog, mapped: &MappedColours, case: &str) {
    let palette = &mapped.palette;
    assert!(
        palette.colours().len() <= MAX_OUTPUT_COLOURS,
        "{case}: more than {MAX_OUTPUT_COLOURS} output colours"
    );
    assert_eq!(
        mapped.similarity.colours(),
        detection.palette.colours().len(),
        "{case}: the similarity tree must cover every detected colour"
    );
    let in_palette = |id: OutputColourId| palette.get(id).is_some();
    assert_eq!(
        mapped.mapping.targets.len(),
        detection.palette.colours().len(),
        "{case}: one mapping target per detected colour"
    );
    assert!(
        mapped.mapping.targets.iter().all(|&t| in_palette(t)),
        "{case}: every mapping target must be in the palette"
    );
    let (src, out) = (&detection.labels, &mapped.labels);
    assert_eq!(
        (out.width(), out.height(), out.upscale()),
        (src.width(), src.height(), src.upscale()),
        "{case}: the output label map must match the detected one in size"
    );
    for (i, (&d, &o)) in src.labels().iter().zip(out.labels()).enumerate() {
        let expected = if d.is_outside() {
            Label::OUTSIDE
        } else {
            match mapped.mapping.target(d) {
                Some(id) => Label::from(id),
                None => panic!("{case}: cell {i} has detected label {} with no target", d.0),
            }
        };
        assert_eq!(
            o, expected,
            "{case}: cell {i} must hold its detected label's target"
        );
    }
    assert!(
        mapped.paints.iter().all(|p| in_palette(p.colour)),
        "{case}: every paint colour must be in the palette"
    );
    let positions: Vec<usize> = mapped.dropped_edits.iter().map(|d| d.position).collect();
    assert!(
        positions.windows(2).all(|w| w[0] < w[1]),
        "{case}: dropped edits must be in log order, once each"
    );
    assert!(
        positions.iter().all(|&p| p < edits.position()),
        "{case}: only active operations can be dropped"
    );
    assert_eq!(
        palette.base().is_some(),
        !palette.colours().is_empty(),
        "{case}: there must be a base colour whenever there are output colours"
    );
}
