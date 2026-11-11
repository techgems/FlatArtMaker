//! Behaviour tests for [`MergeMapper`], plus the port's conformance suite.
//!
//! The fixture ([`fixtures::detection`]) maps automatically to three outputs
//! in slot order: 0 = black (black and near-black merged), 1 = red, 2 = white.

use crate::MergeMapper;
use flatart_domain::{
    ColourName, DetectedColour, DetectedPalette, Detection, DropReason, EditLog, EditOp, Label,
    MAX_OUTPUT_COLOURS, MappedColours, Oklab, OutputColourId, Paint, PxPoint, Rgb,
};
use flatart_ports::ColourMapper;
use flatart_ports::testkit::{colour_mapper, fixtures};

const BLACK: OutputColourId = OutputColourId(0);
const RED: OutputColourId = OutputColourId(1);
const WHITE: OutputColourId = OutputColourId(2);
const ADDED: OutputColourId = OutputColourId::FIRST_ADDED;
const BLUE: Rgb = Rgb {
    r: 30,
    g: 80,
    b: 220,
};

fn log(ops: impl IntoIterator<Item = EditOp>) -> EditLog {
    let mut log = EditLog::new();
    ops.into_iter().for_each(|op| log.push(op));
    log
}

fn map_on(detection: &Detection, edits: &EditLog) -> MappedColours {
    let mapped = MergeMapper
        .map(detection, edits, &fixtures::mapping_settings(3))
        .unwrap();
    colour_mapper::check(detection, edits, &mapped, "test");
    mapped
}

fn map(ops: impl IntoIterator<Item = EditOp>) -> MappedColours {
    map_on(&fixtures::detection(), &log(ops))
}

fn detected(label: usize) -> Oklab {
    fixtures::detection().palette.colours()[label].colour
}

fn target(m: &MappedColours, label: u16) -> OutputColourId {
    m.mapping.target(Label(label)).unwrap()
}

fn report(m: &MappedColours) -> Vec<(usize, DropReason)> {
    m.dropped_edits
        .iter()
        .map(|d| (d.position, d.reason))
        .collect()
}

fn ids(m: &MappedColours) -> Vec<OutputColourId> {
    m.palette.colours().iter().map(|c| c.id).collect()
}

fn name(m: &MappedColours, id: OutputColourId) -> &str {
    m.palette.get(id).unwrap().name.as_str()
}

fn paint(x: f64, y: f64, colour: OutputColourId) -> EditOp {
    EditOp::PaintRegion(Paint {
        at: PxPoint::new(x, y),
        colour,
    })
}

fn add(id: OutputColourId, display: Rgb) -> EditOp {
    EditOp::AddOutput { id, display }
}

fn rename(id: OutputColourId, name: &str) -> EditOp {
    EditOp::Rename {
        id,
        name: ColourName::new(name).unwrap(),
    }
}

#[test]
fn passes_the_conformance_suite() {
    colour_mapper::run(&MergeMapper);
}

#[test]
fn the_automatic_pass_names_colours_and_picks_a_base() {
    let m = map([]);
    assert_eq!(ids(&m), [BLACK, RED, WHITE]);
    let names: Vec<&str> = m
        .palette
        .colours()
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, ["Black", "Red", "White"]);
    assert_eq!(m.palette.base(), Some(BLACK));
    assert!(m.dropped_edits.is_empty() && m.paints.is_empty());
}

#[test]
fn map_points_a_detected_colour_at_an_output() {
    let m = map([EditOp::Map {
        detected: detected(2),
        output: RED,
    }]);
    assert_eq!(target(&m, 2), RED);
    assert_eq!(m.labels.get(0, 2), Some(Label::from(RED)));
}

#[test]
fn paint_is_passed_through_or_dropped() {
    let m = map([
        paint(1.5, 2.5, RED),
        paint(4.0, 0.0, RED),
        paint(-0.5, 1.0, RED),
        paint(1.0, 1.0, OutputColourId(9)),
    ]);
    assert_eq!(m.paints.len(), 1);
    assert_eq!(
        report(&m),
        [
            (1, DropReason::OutsideImage),
            (2, DropReason::OutsideImage),
            (3, DropReason::UnknownOutputColour)
        ]
    );
}

#[test]
fn add_output_appends_a_named_colour() {
    let m = map([
        add(ADDED, BLUE),
        add(OutputColourId(5), BLUE),
        add(ADDED, BLUE),
    ]);
    assert_eq!(ids(&m), [BLACK, RED, WHITE, ADDED]);
    assert_eq!(name(&m, ADDED), "Blue");
    assert_eq!(
        report(&m),
        [
            (1, DropReason::OutputIdUnavailable),
            (2, DropReason::OutputIdUnavailable)
        ]
    );
}

#[test]
fn change_output_colour_keeps_the_mapping_and_renames_unless_named_by_hand() {
    let before = map([]);
    let m = map([
        EditOp::ChangeOutputColour {
            id: WHITE,
            display: BLUE,
        },
        rename(RED, "Fire"),
        EditOp::ChangeOutputColour {
            id: RED,
            display: BLUE,
        },
    ]);
    assert_eq!(m.mapping, before.mapping);
    assert_eq!(m.labels, before.labels);
    assert_eq!(m.palette.get(WHITE).unwrap().display, BLUE);
    assert_eq!(name(&m, WHITE), "Blue");
    assert_eq!(name(&m, RED), "Fire");
}

#[test]
fn delete_output_moves_everything_to_the_target() {
    let m = map([
        EditOp::SetBase { id: Some(WHITE) },
        paint(0.0, 2.0, WHITE),
        EditOp::DeleteOutput {
            id: WHITE,
            reassign_to: RED,
        },
        EditOp::DeleteOutput {
            id: RED,
            reassign_to: RED,
        },
        EditOp::DeleteOutput {
            id: WHITE,
            reassign_to: RED,
        },
    ]);
    assert_eq!(ids(&m), [BLACK, RED]);
    assert_eq!(target(&m, 2), RED);
    assert_eq!(m.paints[0].colour, RED);
    assert_eq!(m.palette.base(), Some(RED), "the base follows its colour");
    assert_eq!(
        report(&m),
        [
            (3, DropReason::ReassignedToItself),
            (4, DropReason::UnknownOutputColour)
        ]
    );
}

#[test]
fn rename_sets_a_unique_name() {
    let m = map([
        rename(RED, "Black"),
        rename(WHITE, "BLACK"),
        rename(OutputColourId(9), "Sky"),
    ]);
    assert_eq!(name(&m, RED), "Black");
    assert_eq!(
        name(&m, BLACK),
        "Black2",
        "automatic names avoid hand-made ones"
    );
    assert_eq!(name(&m, WHITE), "White");
    assert_eq!(
        report(&m),
        [
            (1, DropReason::NameTaken),
            (2, DropReason::UnknownOutputColour)
        ]
    );
}

#[test]
fn set_base_overrides_the_automatic_choice_until_cleared() {
    assert_eq!(
        map([EditOp::SetBase { id: Some(WHITE) }]).palette.base(),
        Some(WHITE)
    );
    let m = map([
        EditOp::SetBase { id: Some(WHITE) },
        EditOp::SetBase { id: None },
        EditOp::SetBase {
            id: Some(OutputColourId(9)),
        },
    ]);
    assert_eq!(m.palette.base(), Some(BLACK));
    assert_eq!(report(&m), [(2, DropReason::UnknownOutputColour)]);
}

#[test]
fn reorder_changes_slots_but_not_names() {
    let m = map([EditOp::Reorder {
        order: vec![WHITE, BLACK, RED],
    }]);
    assert_eq!(ids(&m), [WHITE, BLACK, RED]);
    assert_eq!(m.palette.slot(WHITE, |_| true).unwrap().get(), 1);
    assert_eq!(name(&m, BLACK), "Black");

    let m = map([
        EditOp::Reorder {
            order: vec![WHITE, OutputColourId(9)],
        },
        EditOp::Reorder {
            order: vec![RED, RED],
        },
    ]);
    assert_eq!(
        ids(&m),
        [WHITE, BLACK, RED],
        "unlisted keep their order; unknown are ignored"
    );
    assert_eq!(report(&m), [(1, DropReason::RepeatedInOrder)]);
}

#[test]
fn pin_detected_changes_nothing_here() {
    let pin = Oklab::new(0.5, 0.1, 0.1).unwrap();
    assert_eq!(map([EditOp::PinDetected { colour: pin }]), map([]));
}

#[test]
fn reset_mapping_restores_the_automatic_mapping() {
    let edits = [
        EditOp::Map {
            detected: detected(2),
            output: RED,
        },
        paint(0.0, 0.0, RED),
        EditOp::DeleteOutput {
            id: WHITE,
            reassign_to: BLACK,
        },
        rename(RED, "Fire"),
    ];
    let auto = map([]);
    let kept = map(edits
        .clone()
        .into_iter()
        .chain([EditOp::ResetMapping { clear_paint: false }]));
    assert_eq!(kept.mapping, auto.mapping);
    assert_eq!(kept.paints.len(), 1, "paint is kept");
    assert!(kept.palette.get(WHITE).is_some(), "deleted colour is back");
    assert_eq!(name(&kept, RED), "Fire", "names are kept");
    let cleared = map(edits
        .into_iter()
        .chain([EditOp::ResetMapping { clear_paint: true }]));
    assert!(cleared.paints.is_empty());
}

#[test]
fn reset_is_dropped_when_restoring_would_pass_the_limit() {
    let mut ops = vec![EditOp::DeleteOutput {
        id: WHITE,
        reassign_to: BLACK,
    }];
    ops.extend((0..14).map(|i| add(OutputColourId(ADDED.0 + i), BLUE)));
    ops.push(EditOp::ResetMapping { clear_paint: false });
    let m = map(ops);
    assert_eq!(m.palette.colours().len(), MAX_OUTPUT_COLOURS);
    assert_eq!(report(&m), [(15, DropReason::TooManyOutputColours)]);
}

#[test]
fn sixteen_colours_at_most() {
    let ops = (0..16).map(|i| add(OutputColourId(ADDED.0 + i), BLUE));
    let m = map(ops);
    assert_eq!(m.palette.colours().len(), MAX_OUTPUT_COLOURS);
    assert_eq!(m.dropped_edits.len(), 3);
    let names: std::collections::HashSet<String> = m
        .palette
        .colours()
        .iter()
        .map(|c| c.name.as_str().to_ascii_lowercase())
        .collect();
    assert_eq!(names.len(), MAX_OUTPUT_COLOURS, "names are unique");
    assert_eq!(name(&m, OutputColourId(ADDED.0 + 1)), "Blue2");
}

#[test]
fn undo_and_redo_replay_only_the_active_operations() {
    let first = EditOp::Map {
        detected: detected(2),
        output: RED,
    };
    let mut edits = log([first.clone(), rename(RED, "Fire"), paint(0.0, 0.0, RED)]);
    let full = map_on(&fixtures::detection(), &edits);
    edits.undo();
    edits.undo();
    assert_eq!(map_on(&fixtures::detection(), &edits), map([first]));
    edits.redo();
    edits.redo();
    assert_eq!(map_on(&fixtures::detection(), &edits), full);
}

/// The fixture after a settings change: every colour shifted slightly, and
/// white no longer detected.
fn shifted_detection() -> Detection {
    let colour = |l, a, b, pixels| DetectedColour {
        colour: Oklab::new(l, a, b).unwrap(),
        pixels,
    };
    Detection {
        palette: DetectedPalette::new(vec![
            colour(0.21, 0.01, 0.0, 10),
            colour(0.60, 0.20, 0.12, 5),
        ])
        .unwrap(),
        labels: fixtures::labels(&["0001", "0011", "0011", ".111"], 1),
    }
}

#[test]
fn edits_are_rematched_after_detection_changes() {
    let edits = log([
        EditOp::Map {
            detected: detected(0),
            output: RED,
        },
        EditOp::Map {
            detected: detected(2),
            output: RED,
        },
    ]);
    let m = map_on(&shifted_detection(), &edits);
    assert_eq!(target(&m, 0), RED, "black still matches its shifted colour");
    assert_eq!(
        report(&m),
        [(1, DropReason::NoMatchingDetectedColour)],
        "white is gone, so its edit is reported"
    );
}

/// SplitMix64, for repeatable random logs.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn id(&mut self) -> OutputColourId {
        const POOL: [u16; 8] = [0, 1, 2, 3, 16, 17, 18, 99];
        OutputColourId(POOL[self.below(POOL.len())])
    }

    fn rgb(&mut self) -> Rgb {
        let v = self.next().to_le_bytes();
        Rgb {
            r: v[0],
            g: v[1],
            b: v[2],
        }
    }

    fn oklab(&mut self) -> Oklab {
        match self.below(3) {
            0 => detected(self.below(4)),
            1 => {
                let c = detected(self.below(4));
                Oklab::new(c.l() + 0.01, c.a(), c.b()).unwrap()
            }
            _ => Oklab::new(self.below(100) as f64 / 100.0, 0.1, -0.1).unwrap(),
        }
    }

    fn op(&mut self) -> EditOp {
        const NAMES: [&str; 5] = ["Black", "black", "Sun", "Moon", "White2"];
        match self.below(10) {
            0 => EditOp::Map {
                detected: self.oklab(),
                output: self.id(),
            },
            1 => paint(
                self.below(6) as f64 - 1.0,
                self.below(6) as f64 - 1.0,
                self.id(),
            ),
            2 => add(self.id(), self.rgb()),
            3 => EditOp::ChangeOutputColour {
                id: self.id(),
                display: self.rgb(),
            },
            4 => EditOp::DeleteOutput {
                id: self.id(),
                reassign_to: self.id(),
            },
            5 => rename(self.id(), NAMES[self.below(NAMES.len())]),
            6 => EditOp::SetBase {
                id: (self.below(2) == 0).then(|| self.id()),
            },
            7 => EditOp::Reorder {
                order: (0..self.below(5)).map(|_| self.id()).collect(),
            },
            8 => EditOp::PinDetected {
                colour: self.oklab(),
            },
            _ => EditOp::ResetMapping {
                clear_paint: self.below(2) == 0,
            },
        }
    }
}

#[test]
fn replaying_any_log_twice_gives_the_same_valid_result() {
    let mut rng = Rng(42);
    for case in 0..500 {
        let len = rng.below(40);
        let mut edits = log((0..len).map(|_| rng.op()));
        for _ in 0..rng.below(4) {
            edits.undo();
        }
        for detection in [fixtures::detection(), shifted_detection()] {
            // `map_on` also checks every contract clause.
            let first = map_on(&detection, &edits);
            let second = map_on(&detection, &edits);
            assert_eq!(first, second, "case {case}: replay is not deterministic");
            let unedited = map_on(&detection, &EditLog::new());
            assert_eq!(
                first.similarity, unedited.similarity,
                "case {case}: edits must not change the similarity tree"
            );
        }
    }
}
