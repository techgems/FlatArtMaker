//! Records `fixtures/jolteon/edits.json`, the edit log the golden tests
//! replay: Jolteon's orange gets its own output colour (mapping), and the
//! tongue is painted pink (painting).
//!
//! `cargo run -p flatart-cli --example make_jolteon_edits`

use flatart_cli::{Job, read_png};
use flatart_domain::{EditLog, EditOp, Label, Paint, PxPoint, Rgb, Rgba, RgbaImage};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/jolteon");

/// The pixels closest to `target`, by squared RGB distance, as (x, y).
fn closest(image: &RgbaImage, target: (i32, i32, i32), within: i32) -> Vec<(u32, u32)> {
    let mut found = Vec::new();
    for y in 0..image.height() {
        for x in 0..image.width() {
            let Some(Rgba { r, g, b, a }) = image.get(x, y) else {
                continue;
            };
            let d = (i32::from(r) - target.0).pow(2)
                + (i32::from(g) - target.1).pow(2)
                + (i32::from(b) - target.2).pow(2);
            if a == 255 && d <= within * within {
                found.push((x, y));
            }
        }
    }
    found
}

fn main() {
    let image = read_png(&std::fs::read(format!("{DIR}/jolteon.png")).unwrap()).unwrap();
    let mut pipeline = Job::new(image.clone()).pipeline().unwrap();
    let detection = pipeline.detection().unwrap().clone();
    let label_at = |(x, y)| detection.labels.get(x, y).unwrap();

    // The detected colour most of Jolteon's orange pixels have.
    let orange_pixels = closest(&image, (240, 150, 40), 30);
    let mut counts = std::collections::BTreeMap::<u16, usize>::new();
    for &p in &orange_pixels {
        *counts.entry(label_at(p).0).or_default() += 1;
    }
    let (&orange, _) = counts
        .iter()
        .max_by_key(|(_, n)| **n)
        .expect("orange pixels");
    let orange_colour = detection.palette.colours()[usize::from(orange)].colour;

    // A pink pixel near the middle of the tongue.
    let pink = closest(&image, (235, 150, 170), 25);
    let mid = pink[pink.len() / 2];
    assert_ne!(label_at(mid), Label::OUTSIDE);

    let mut edits = EditLog::new();
    let orange_id = edits.next_output_id();
    edits.push(EditOp::AddOutput {
        id: orange_id,
        display: Rgb {
            r: 240,
            g: 150,
            b: 40,
        },
    });
    edits.push(EditOp::Map {
        detected: orange_colour,
        output: orange_id,
    });
    let pink_id = edits.next_output_id();
    edits.push(EditOp::AddOutput {
        id: pink_id,
        display: Rgb {
            r: 235,
            g: 150,
            b: 170,
        },
    });
    edits.push(EditOp::PaintRegion(Paint {
        at: PxPoint::new(f64::from(mid.0) + 0.5, f64::from(mid.1) + 0.5),
        colour: pink_id,
    }));
    let json = serde_json::to_string_pretty(&edits).unwrap();
    std::fs::write(format!("{DIR}/edits.json"), json + "\n").unwrap();
    println!(
        "orange: detected colour {orange} ({} px); tongue at {mid:?} ({} pink px)",
        orange_pixels.len(),
        pink.len()
    );
}
