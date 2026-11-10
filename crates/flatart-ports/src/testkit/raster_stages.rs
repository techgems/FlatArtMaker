//! Conformance suite for [`RasterStages`].

use super::{fixtures, ok};
use crate::RasterStages;
use flatart_domain::settings::{
    CleanUpSettings, LoadSettings, MinFeatureWidth, PrintScale, PrintabilitySettings, SpeckleArea,
    TraceUpscale,
};
use flatart_domain::{
    ArtBounds, Label, LabelMap, Mm, MmPerPx, OutputColourId, Paint, PxPoint, Rgb, Rgba, RgbaImage,
};
use std::collections::BTreeSet;

/// Runs every check on `stages`.
pub fn run(stages: &dyn RasterStages) {
    load(stages);
    clean_up(stages);
    paint(stages);
    printability(stages);
}

fn load(stages: &dyn RasterStages) {
    let defaults = LoadSettings::default();
    let image = fixtures::two_colour_image();
    let loaded = ok(stages.load(&image, &defaults), "load");
    assert_eq!(
        loaded.bounds,
        ArtBounds {
            x: 1,
            y: 1,
            width: 4,
            height: 4
        },
        "load: the bounds must ignore the transparent margin"
    );
    for (i, (&before, &after)) in image.pixels().iter().zip(loaded.image.pixels()).enumerate() {
        let expected = if before.a < defaults.alpha_threshold {
            Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }
        } else {
            Rgba { a: 255, ..before }
        };
        assert_eq!(after, expected, "load: pixel {i}");
    }
    assert_eq!(
        stages.load(&image, &defaults).ok(),
        Some(loaded),
        "load: not deterministic"
    );

    // Alpha just below the threshold is outside; at the threshold it isn't.
    let faint = |a| Rgba { a, ..fixtures::RED };
    let image = RgbaImage::new(2, 1, vec![faint(127), faint(128)]).expect("valid image");
    let loaded = ok(stages.load(&image, &defaults), "load alpha");
    assert_eq!(
        (loaded.image.pixels()[0].a, loaded.image.pixels()[1].a),
        (0, 255),
        "load alpha: alpha below the threshold must be outside, and only that"
    );

    // A background colour marks outside in an opaque image.
    let opaque = RgbaImage::new(3, 1, vec![fixtures::RED, fixtures::BLUE, fixtures::RED])
        .expect("valid image");
    let red = Rgb {
        r: fixtures::RED.r,
        g: fixtures::RED.g,
        b: fixtures::RED.b,
    };
    let settings = LoadSettings {
        background: Some(red),
        ..defaults
    };
    let loaded = ok(stages.load(&opaque, &settings), "load background");
    assert_eq!(
        loaded.bounds,
        ArtBounds {
            x: 1,
            y: 0,
            width: 1,
            height: 1
        },
        "load background: pixels of the background colour must be outside"
    );

    let empty = fixtures::image(&["..", ".."]);
    assert!(
        stages.load(&empty, &defaults).is_err(),
        "load: an image with nothing but outside must fail"
    );
}

fn speckle(cells: u32) -> CleanUpSettings {
    CleanUpSettings {
        speckle_area: SpeckleArea::new(cells).expect("valid speckle area"),
    }
}

fn clean_up(stages: &dyn RasterStages) {
    for (name, map) in fixtures::all_label_maps() {
        let case = format!("clean up {name}");
        let out = ok(stages.clean_up(&map, &speckle(16)), &case);
        check_same_size(&map, &out, &case);
        check_outside_kept(&map, &out, &case);
        check_labels_from(&map, &out, &[], &case);
        assert_eq!(
            stages.clean_up(&map, &speckle(16)).ok(),
            Some(out),
            "{case}: not deterministic"
        );
    }

    let lone = fixtures::labels(&["00000", "00000", "00100", "00000", "00000"], 1);
    let out = ok(stages.clean_up(&lone, &speckle(0)), "mode filter");
    assert!(
        out.labels().iter().all(|&l| l == Label(0)),
        "mode filter: a lone cell must take its surroundings' label"
    );

    let block = fixtures::labels(
        &[
            "00000000", "00000000", "00111100", "00111100", "00111100", "00111100", "00000000",
            "00000000",
        ],
        1,
    );
    let kept = ok(stages.clean_up(&block, &speckle(0)), "speckle off");
    assert!(
        kept.labels().contains(&Label(1)),
        "speckle off: a 4 × 4 block must survive"
    );
    let merged = ok(stages.clean_up(&block, &speckle(500)), "speckle on");
    assert!(
        merged.labels().iter().all(|&l| l == Label(0)),
        "speckle on: an area smaller than the speckle area must merge into its neighbour"
    );

    let island = fixtures::labels(&["000..", "000..", "000.1"], 1);
    let out = ok(stages.clean_up(&island, &speckle(500)), "outside island");
    assert_eq!(
        out.get(4, 2),
        Some(Label(1)),
        "outside island: an area bordering only the outside must be kept"
    );
}

fn paint_at(x: f64, y: f64, colour: u16) -> Paint {
    Paint {
        at: PxPoint::new(x, y),
        colour: OutputColourId(colour),
    }
}

fn paint(stages: &dyn RasterStages) {
    let map = fixtures::three_labels();
    let painted = ok(stages.paint(&map, &[paint_at(0.5, 0.5, 2)]), "paint");
    check_same_size(&map, &painted, "paint");
    let expected = fixtures::labels(&["2211", "2211", "2222"], 1);
    assert_eq!(painted, expected, "paint: only the clicked area may change");

    let paints = [paint_at(0.5, 0.5, 7), paint_at(0.5, 0.5, 9)];
    let painted = ok(stages.paint(&map, &paints), "paint in order");
    assert_eq!(
        painted.get(0, 0),
        Some(Label(9)),
        "paint in order: the later paint must win"
    );
    check_labels_from(&map, &painted, &[7, 9], "paint in order");

    let nested = fixtures::nested_labels();
    for (at, case) in [
        ((0.5, 0.5), "paint outside"),
        ((-1.0, 9.0), "paint off image"),
    ] {
        let painted = ok(stages.paint(&nested, &[paint_at(at.0, at.1, 1)]), case);
        assert_eq!(painted, nested, "{case}: must change nothing");
    }

    let doubled = fixtures::upscaled(&nested, 2);
    let painted = ok(
        stages.paint(&doubled, &[paint_at(2.5, 2.5, 0)]),
        "paint upscaled",
    );
    assert!(
        !painted.labels().contains(&Label(1)),
        "paint upscaled: the point is in source pixels, so (2.5, 2.5) is in the island"
    );
}

fn printable_settings(upscale: u8, mm_per_px: f64, min: Option<f64>) -> PrintabilitySettings {
    PrintabilitySettings {
        upscale: TraceUpscale::new(upscale).expect("valid upscale"),
        scale: PrintScale::new(MmPerPx(mm_per_px)).expect("valid scale"),
        min_feature: min.map(|w| MinFeatureWidth::new(Mm(w)).expect("valid width")),
    }
}

fn printability(stages: &dyn RasterStages) {
    let nested = fixtures::nested_labels();
    let out = ok(
        stages.printability(&nested, &printable_settings(2, 1.0, None)),
        "upscale",
    );
    assert_eq!(
        out.labels,
        fixtures::upscaled(&nested, 2),
        "upscale: must repeat each cell"
    );
    assert!(out.thin_features.is_empty(), "upscale: the fix is off");
    let doubled = fixtures::upscaled(&nested, 2);
    assert!(
        stages
            .printability(&doubled, &printable_settings(3, 1.0, None))
            .is_err(),
        "upscale: ×2 to ×3 isn't a whole multiple and must fail"
    );

    // A 1 px line at 0.2 mm/px is 0.2 mm wide: thinner than 0.25 mm.
    let line = fixtures::labels(&["0001000"; 7], 1);
    for upscale in [1, 2] {
        let case = format!("thin line ×{upscale}");
        let fixed = ok(
            stages.printability(&line, &printable_settings(upscale, 0.2, Some(0.25))),
            &case,
        );
        assert!(
            fixed.labels.labels().iter().all(|&l| l == Label(0)),
            "{case}: a line thinner than the minimum must go"
        );
        check_labels_from(&line, &fixed.labels, &[], &case);
        let off = ok(
            stages.printability(&line, &printable_settings(upscale, 0.2, None)),
            &case,
        );
        assert_eq!(
            off.labels,
            fixtures::upscaled(&line, upscale),
            "{case}: with the fix off the line must stay"
        );
        assert!(off.thin_features.is_empty(), "{case}: the fix is off");
    }

    // A 2 px line at 0.2 mm/px is 0.4 mm wide: wide enough.
    let wide = fixtures::labels(&["00011000"; 8], 1);
    let fixed = ok(
        stages.printability(&wide, &printable_settings(1, 0.2, Some(0.25))),
        "wide line",
    );
    assert_eq!(
        fixed.labels, wide,
        "wide line: parts at least the minimum wide must not change"
    );
    assert!(
        fixed.thin_features.is_empty(),
        "wide line: nothing is too thin"
    );

    for (name, map) in fixtures::all_label_maps() {
        let case = format!("printability {name}");
        let settings = printable_settings(4, 0.2, Some(0.5));
        let out = ok(stages.printability(&map, &settings), &case);
        check_labels_from(&map, &out.labels, &[], &case);
        assert_eq!(
            stages.printability(&map, &settings).ok(),
            Some(out),
            "{case}: not deterministic"
        );
    }
}

fn check_same_size(before: &LabelMap, after: &LabelMap, case: &str) {
    assert_eq!(
        (after.width(), after.height(), after.upscale()),
        (before.width(), before.height(), before.upscale()),
        "{case}: the label map must keep its size"
    );
}

fn check_outside_kept(before: &LabelMap, after: &LabelMap, case: &str) {
    for (i, (&b, &a)) in before.labels().iter().zip(after.labels()).enumerate() {
        assert_eq!(
            b.is_outside(),
            a.is_outside(),
            "{case}: cell {i} changed between outside and inside"
        );
    }
}

/// Every output label was in the input, or is one of `added`.
fn check_labels_from(before: &LabelMap, after: &LabelMap, added: &[u16], case: &str) {
    let allowed: BTreeSet<Label> = before
        .labels()
        .iter()
        .copied()
        .chain(added.iter().map(|&l| Label(l)))
        .collect();
    let stray: BTreeSet<Label> = after
        .labels()
        .iter()
        .copied()
        .filter(|l| !allowed.contains(l))
        .collect();
    assert!(
        stray.is_empty(),
        "{case}: labels {stray:?} weren't in the input"
    );
}
