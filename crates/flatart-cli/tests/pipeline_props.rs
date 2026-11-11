//! Property tests across the whole wired pipeline: random flat-colour
//! images, random settings, every exporter.

use flatart_cli::{Job, ThinFix};
use flatart_domain::{ExportOptions, PartKind, Rgba, RgbaImage};
use flatart_ports::testkit::mesh::assert_model_watertight;
use proptest::prelude::*;

const COLOURS: [Rgba; 6] = [
    Rgba {
        r: 20,
        g: 20,
        b: 20,
        a: 255,
    },
    Rgba {
        r: 245,
        g: 240,
        b: 225,
        a: 255,
    },
    Rgba {
        r: 240,
        g: 200,
        b: 30,
        a: 255,
    },
    Rgba {
        r: 235,
        g: 120,
        b: 30,
        a: 255,
    },
    Rgba {
        r: 40,
        g: 90,
        b: 200,
        a: 255,
    },
    Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    },
];

/// Blocky images: random colours from `COLOURS` (the last is transparent)
/// in blocks of 1 to 3 pixels, with at least one opaque pixel.
fn image() -> impl Strategy<Value = RgbaImage> {
    (3u32..14, 3u32..14, 1u32..4).prop_flat_map(|(w, h, block)| {
        let cols = w.div_ceil(block);
        let cells = (cols * h.div_ceil(block)) as usize;
        proptest::collection::vec(0usize..COLOURS.len(), cells)
            .prop_filter("needs an opaque pixel", |c| {
                c.iter().any(|&i| i + 1 < COLOURS.len())
            })
            .prop_map(move |cells| {
                let pixels = (0..h)
                    .flat_map(|y| (0..w).map(move |x| (x, y)))
                    .map(|(x, y)| COLOURS[cells[((y / block) * cols + x / block) as usize]])
                    .collect();
                RgbaImage::new(w, h, pixels).unwrap()
            })
    })
}

fn job() -> impl Strategy<Value = Job> {
    (
        image(),
        1u8..7,
        prop_oneof![Just(0.0), Just(2.0)],
        prop_oneof![
            Just(ThinFix::Off),
            Just(ThinFix::MinWidth(0.25)),
            Just(ThinFix::MinWidth(1.0))
        ],
        20.0f64..120.0,
    )
        .prop_map(|(image, output_colours, base, thin_fix, size_x)| Job {
            output_colours,
            base,
            thin_fix,
            size_x,
            ..Job::new(image)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn every_image_gives_watertight_parts_and_the_same_files_twice(job in job()) {
        let mut pipeline = job.pipeline().unwrap();
        let model = pipeline.mesh().unwrap().clone();
        assert_model_watertight(&model);
        let has_base = model.parts.iter().any(|p| p.kind == PartKind::Base);
        prop_assert_eq!(has_base, job.base > 0.0 && !model.parts.is_empty());

        let options = ExportOptions {
            name: "random".into(),
            thumbnail: Some(pipeline.thumbnail(64).unwrap()),
        };
        let mut again = job.pipeline().unwrap();
        for id in ["3mf-prusa", "3mf-bambu", "svg"] {
            let bytes = pipeline.export(id, &options).unwrap();
            prop_assert!(!bytes.is_empty());
            prop_assert!(bytes == again.export(id, &options).unwrap(), "{} isn't deterministic", id);
        }
    }
}
