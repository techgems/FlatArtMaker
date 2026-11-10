//! Conformance suite for [`ColorQuantizer`].

use super::{fixtures, ok};
use crate::ColorQuantizer;
use flatart_domain::settings::{DetectedColourLimit, QuantSettings};
use flatart_domain::{Detection, Label, Oklab, RgbaImage};
use std::collections::HashMap;

/// Runs every check on `quantizer`.
pub fn run(quantizer: &dyn ColorQuantizer) {
    let image = fixtures::two_colour_image();
    let settings = QuantSettings {
        max_colours: DetectedColourLimit::new(8).expect("valid limit"),
    };

    let detection = ok(quantizer.quantize(&image, &settings, &[]), "two colours");
    check(&image, &detection, &settings, 0, "two colours");
    let red = label_at(&detection, 1, 1);
    let blue = label_at(&detection, 2, 2);
    assert_ne!(
        red, blue,
        "two colours: red and blue must get different labels when the limit allows"
    );
    let again = ok(quantizer.quantize(&image, &settings, &[]), "two colours");
    assert_eq!(detection, again, "two colours: not deterministic");

    let pin = Oklab::new(0.62, -0.11, 0.09).expect("valid pin");
    let detection = ok(quantizer.quantize(&image, &settings, &[pin]), "pinned");
    check(&image, &detection, &settings, 1, "pinned");
    assert!(
        detection.palette.colours().iter().any(|c| c.colour == pin),
        "pinned: the pin must appear in the palette with exactly its value"
    );
}

fn label_at(detection: &Detection, x: u32, y: u32) -> Label {
    detection
        .labels
        .get(x, y)
        .unwrap_or_else(|| panic!("label map has no cell ({x}, {y})"))
}

fn check(
    image: &RgbaImage,
    detection: &Detection,
    settings: &QuantSettings,
    pins: usize,
    case: &str,
) {
    let (labels, palette) = (&detection.labels, &detection.palette);
    assert_eq!(
        (labels.width(), labels.height(), labels.upscale().get()),
        (image.width(), image.height(), 1),
        "{case}: label map must match the image size, with upscale 1"
    );
    let max = usize::from(settings.max_colours.get()) + pins;
    assert!(
        palette.colours().len() <= max,
        "{case}: {} colours is over the limit of {max}",
        palette.colours().len()
    );

    let mut counts = vec![0u64; palette.colours().len()];
    let mut by_rgb: HashMap<(u8, u8, u8), Label> = HashMap::new();
    for (i, (pixel, &label)) in image.pixels().iter().zip(labels.labels()).enumerate() {
        if pixel.a == 0 {
            assert!(
                label.is_outside(),
                "{case}: transparent pixel {i} must be outside"
            );
            continue;
        }
        let index = usize::from(label.0);
        assert!(
            !label.is_outside() && index < counts.len(),
            "{case}: pixel {i} has label {} with no palette entry",
            label.0
        );
        counts[index] += 1;
        let first = *by_rgb.entry((pixel.r, pixel.g, pixel.b)).or_insert(label);
        assert_eq!(
            first, label,
            "{case}: pixels of the same colour got different labels"
        );
    }
    for (i, (colour, count)) in palette.colours().iter().zip(counts).enumerate() {
        assert_eq!(
            colour.pixels, count,
            "{case}: palette entry {i} says {} pixels but {count} have its label",
            colour.pixels
        );
    }
}
