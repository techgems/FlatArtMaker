use crate::PixelStages;
use crate::clean_up::merge_speckles;
use crate::grid::Grid;
use flatart_domain::settings::{
    CleanUpSettings, LoadSettings, MinFeatureWidth, PrintScale, PrintabilitySettings, SpeckleArea,
    TraceUpscale,
};
use flatart_domain::{Label, LabelMap, Mm, MmPerPx, OutputColourId, Paint, PxPoint};
use flatart_ports::RasterStages;
use flatart_ports::testkit::{fixtures, raster_stages};
use std::collections::BTreeSet;

#[test]
fn conforms() {
    raster_stages::run(&PixelStages);
}

fn settings(upscale: u8, mm_per_px: f64, min: Option<f64>) -> PrintabilitySettings {
    PrintabilitySettings {
        upscale: TraceUpscale::new(upscale).unwrap(),
        scale: PrintScale::new(MmPerPx(mm_per_px)).unwrap(),
        min_feature: min.map(|w| MinFeatureWidth::new(Mm(w)).unwrap()),
    }
}

/// Jolteon at the default 100 mm: about 0.22 mm per source pixel.
const JOLTEON_SCALE: f64 = 100.0 / 451.0;

fn rows(map: &LabelMap) -> Vec<String> {
    (0..map.height())
        .map(|y| {
            (0..map.width())
                .map(|x| match map.get(x, y).unwrap() {
                    l if l.is_outside() => '.',
                    l => char::from_digit(u32::from(l.0), 10).unwrap_or('?'),
                })
                .collect()
        })
        .collect()
}

#[test]
fn a_one_pixel_line_goes_at_a_quarter_millimetre_only_when_the_fix_is_on() {
    let line = fixtures::labels(&["0000100000"; 10], 1);
    for upscale in 1..=4 {
        let on = PixelStages
            .printability(&line, &settings(upscale, JOLTEON_SCALE, Some(0.25)))
            .unwrap();
        assert!(
            !on.labels.labels().contains(&Label(1)),
            "×{upscale}: the line must go"
        );
        let off = PixelStages
            .printability(&line, &settings(upscale, JOLTEON_SCALE, None))
            .unwrap();
        assert_eq!(off.labels, fixtures::upscaled(&line, upscale));
    }
}

#[test]
fn a_thin_island_goes_to_the_colour_around_it() {
    let map = fixtures::labels(
        &["00000000", "00000000", "00011000", "00000000", "00000000"],
        1,
    );
    let out = PixelStages
        .printability(&map, &settings(1, 0.2, Some(0.6)))
        .unwrap();
    assert!(out.labels.labels().iter().all(|&l| l == Label(0)));
    assert!(out.thin_features.is_empty());
}

#[test]
fn a_thin_line_between_two_colours_goes_and_both_sides_stay() {
    let map = fixtures::labels(&["00000122222"; 6], 1);
    let out = PixelStages
        .printability(&map, &settings(1, 0.2, Some(0.25)))
        .unwrap();
    assert!(!out.labels.labels().contains(&Label(1)));
    // The two sides each keep their own cells.
    assert_eq!(rows(&out.labels)[0][..5], *"00000");
    assert_eq!(rows(&out.labels)[0][6..], *"22222");
}

#[test]
fn a_whisker_into_the_outside_is_kept_and_reported() {
    let map = fixtures::labels(
        &[
            "000000....",
            "000000....",
            "0000000000",
            "000000....",
            "000000....",
        ],
        1,
    );
    let out = PixelStages
        .printability(&map, &settings(1, 0.2, Some(0.4)))
        .unwrap();
    assert_eq!(out.labels, map, "the outline must not change");
    assert_eq!(out.thin_features.len(), 1, "{:?}", out.thin_features);
    let feature = out.thin_features[0];
    assert_eq!(feature.colour, OutputColourId(0));
    assert_eq!(feature.at.y.0, 2.5);
    assert!(feature.at.x.0 > 6.0);
}

#[test]
fn rounded_corners_of_wide_parts_are_not_reported() {
    let mut map_rows = vec!["................"; 2];
    map_rows.extend(vec!["..000000000000.."; 12]);
    map_rows.extend(vec!["................"; 2]);
    let map = fixtures::labels(&map_rows, 1);
    // A 6-cell disc: corners can't be reached, but they're rounding.
    let out = PixelStages
        .printability(&map, &settings(1, 0.1, Some(0.6)))
        .unwrap();
    assert_eq!(out.labels, map);
    assert!(out.thin_features.is_empty(), "{:?}", out.thin_features);
}

#[test]
fn a_speckle_merges_into_the_neighbour_with_the_longest_border() {
    // The 2-cell area of 1 borders 0 on 2 cells and 2 on 4.
    let map = fixtures::labels(&["00222", "00122", "00122", "22222"], 1);
    let mut labels = map.labels().to_vec();
    merge_speckles(Grid::new(5, 4), &mut labels, 3);
    let merged = LabelMap::new(5, 4, map.upscale(), labels).unwrap();
    assert_eq!(rows(&merged), ["00222", "00222", "00222", "22222"]);
}

#[test]
fn merged_speckles_count_toward_their_new_area() {
    // 1 (2 cells) merges into 2 (2 cells), making 4: no longer a speckle,
    // so 2 doesn't then merge into 0.
    let map = fixtures::labels(&["....", ".12.", ".12.", "0000"], 1);
    let mut labels = map.labels().to_vec();
    merge_speckles(Grid::new(4, 4), &mut labels, 3);
    let merged = LabelMap::new(4, 4, map.upscale(), labels).unwrap();
    assert_eq!(rows(&merged), ["....", ".22.", ".22.", "0000"]);
}

#[test]
fn paint_changes_only_the_clicked_area() {
    let map = fixtures::labels(&["0010", "0010", "1111", "0000"], 1);
    let out = PixelStages
        .paint(
            &map,
            &[Paint {
                at: PxPoint::new(0.2, 0.7),
                colour: OutputColourId(5),
            }],
        )
        .unwrap();
    assert_eq!(rows(&out), ["5510", "5510", "1111", "0000"]);
}

#[test]
fn load_marks_the_background_colour_as_outside() {
    let image = fixtures::image(&["rrrr", "rbbr", "rrrr"]);
    let red = flatart_domain::Rgb {
        r: fixtures::RED.r,
        g: fixtures::RED.g,
        b: fixtures::RED.b,
    };
    let loaded = PixelStages
        .load(
            &image,
            &LoadSettings {
                background: Some(red),
                ..LoadSettings::default()
            },
        )
        .unwrap();
    let b = loaded.bounds;
    assert_eq!((b.x, b.y, b.width, b.height), (1, 1, 2, 1));
}

/// A small deterministic generator for the random maps below.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % n
    }
}

fn random_map(rng: &mut Lcg) -> LabelMap {
    let (w, h) = (3 + rng.below(14) as u32, 3 + rng.below(14) as u32);
    // Blocky maps, so there are wide parts as well as noise.
    let block = 1 + rng.below(3) as u32;
    let cols = w.div_ceil(block);
    let cells: Vec<Label> = (0..cols * h.div_ceil(block))
        .map(|_| match rng.below(6) {
            5 => Label::OUTSIDE,
            l => Label(l as u16),
        })
        .collect();
    let labels = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| cells[((y / block) * cols + x / block) as usize])
        .collect();
    LabelMap::new(w, h, TraceUpscale::new(1).unwrap(), labels).unwrap()
}

fn label_set(map: &LabelMap) -> BTreeSet<Label> {
    map.labels().iter().copied().collect()
}

#[test]
fn no_stage_invents_a_label_or_moves_the_outline() {
    let mut rng = Lcg(7);
    for case in 0..300 {
        let map = random_map(&mut rng);
        let input = label_set(&map);
        let speckle = CleanUpSettings {
            speckle_area: SpeckleArea::new(rng.below(20) as u32).unwrap(),
        };
        let cleaned = PixelStages.clean_up(&map, &speckle).unwrap();
        assert!(label_set(&cleaned).is_subset(&input), "case {case}");

        let upscale = 1 + rng.below(4) as u8;
        let min = 0.25 + rng.below(20) as f64 * 0.05;
        let out = PixelStages
            .printability(&cleaned, &settings(upscale, 0.2, Some(min)))
            .unwrap();
        assert!(label_set(&out.labels).is_subset(&input), "case {case}");
        let before = fixtures::upscaled(&cleaned, upscale);
        for (i, (&b, &a)) in before.labels().iter().zip(out.labels.labels()).enumerate() {
            assert_eq!(b.is_outside(), a.is_outside(), "case {case}, cell {i}");
        }
        assert_eq!(
            PixelStages
                .printability(&cleaned, &settings(upscale, 0.2, Some(min)))
                .unwrap(),
            out,
            "case {case}: not deterministic"
        );
    }
}
