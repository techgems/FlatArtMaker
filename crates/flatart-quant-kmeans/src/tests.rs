//! Behaviour tests for [`KmeansQuantizer`], plus the port's conformance suite.

use crate::KmeansQuantizer;
use flatart_domain::settings::{DetectedColourLimit, QuantSettings};
use flatart_domain::{Detection, Label, Oklab, Rgba, RgbaImage};
use flatart_ports::ColorQuantizer;
use flatart_ports::testkit::{color_quantizer, fixtures};

fn settings(max: u8) -> QuantSettings {
    QuantSettings {
        max_colours: DetectedColourLimit::new(max).unwrap(),
    }
}

fn image(width: u32, height: u32, pixel: impl Fn(u32, u32) -> Rgba) -> RgbaImage {
    let pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| pixel(x, y))
        .collect();
    RgbaImage::new(width, height, pixels).unwrap()
}

fn rgba(r: u8, g: u8, b: u8) -> Rgba {
    Rgba { r, g, b, a: 255 }
}

/// 32 × 32 with 1024 distinct colours.
fn gradient() -> RgbaImage {
    image(32, 32, |x, y| rgba((x * 8) as u8, (y * 8) as u8, 128))
}

fn detect(image: &RgbaImage, max: u8, pins: &[Oklab]) -> Detection {
    KmeansQuantizer
        .quantize(image, &settings(max), pins)
        .unwrap()
}

fn label(d: &Detection, x: u32, y: u32) -> Label {
    d.labels.get(x, y).unwrap()
}

#[test]
fn passes_the_conformance_suite() {
    color_quantizer::run(&KmeansQuantizer);
}

#[test]
fn a_two_colour_image_gives_exactly_two_colours() {
    let d = detect(&fixtures::two_colour_image(), 24, &[]);
    assert_eq!(d.palette.colours().len(), 2);
    // 12 red pixels, 4 blue: sorted by coverage.
    let counts: Vec<u64> = d.palette.colours().iter().map(|c| c.pixels).collect();
    assert_eq!(counts, [12, 4]);
}

#[test]
fn transparent_pixels_are_outside_whatever_their_colour() {
    let img = image(4, 1, |x, _| Rgba {
        r: 10 * x as u8,
        g: 200,
        b: 0,
        a: if x % 2 == 0 { 0 } else { 255 },
    });
    let d = detect(&img, 24, &[]);
    assert!(label(&d, 0, 0).is_outside());
    assert!(label(&d, 2, 0).is_outside());
    assert!(!label(&d, 1, 0).is_outside());
    let total: u64 = d.palette.colours().iter().map(|c| c.pixels).sum();
    assert_eq!(total, 2, "transparent pixels must not count");
}

#[test]
fn a_pinned_colour_survives_unchanged() {
    let pin = Oklab::new(0.42, 0.031, -0.077).unwrap();
    for (img, case) in [
        (fixtures::two_colour_image(), "few colours"),
        (gradient(), "k-means"),
    ] {
        let d = detect(&img, 6, &[pin]);
        assert!(
            d.palette.colours().iter().any(|c| c.colour == pin),
            "{case}: pin missing"
        );
        assert!(
            d.palette.colours().len() <= 6,
            "{case}: pins count toward the limit"
        );
    }
}

#[test]
fn pins_beyond_the_limit_are_all_kept() {
    let pins: Vec<Oklab> = [0.2, 0.5, 0.8]
        .iter()
        .map(|&l| Oklab::new(l, 0.0, 0.0).unwrap())
        .collect();
    let d = detect(&gradient(), 2, &pins);
    let found: Vec<Oklab> = d.palette.colours().iter().map(|c| c.colour).collect();
    assert_eq!(found.len(), 3);
    assert!(pins.iter().all(|p| found.contains(p)));
}

#[test]
fn repeated_pins_count_once() {
    let pin = Oklab::new(0.3, 0.0, 0.0).unwrap();
    let d = detect(&gradient(), 4, &[pin, pin]);
    let pinned = d
        .palette
        .colours()
        .iter()
        .filter(|c| c.colour == pin)
        .count();
    assert_eq!(pinned, 1);
}

#[test]
fn k_means_respects_the_limit_and_is_deterministic() {
    let img = gradient();
    let d = detect(&img, 5, &[]);
    assert_eq!(d.palette.colours().len(), 5);
    let counts: Vec<u64> = d.palette.colours().iter().map(|c| c.pixels).collect();
    assert!(
        counts.windows(2).all(|w| w[0] >= w[1]),
        "sorted by coverage"
    );
    assert_eq!(counts.iter().sum::<u64>(), 1024);
    assert_eq!(d, detect(&img, 5, &[]));
}

#[test]
fn noisy_flat_colours_are_found() {
    // Three flat colours with ±2 noise: 3 × 25 = 75 distinct values.
    let base = [rgba(200, 40, 40), rgba(40, 180, 60), rgba(30, 50, 200)];
    let img = image(30, 30, |x, y| {
        let c = base[(x / 10) as usize];
        let n = |v: u8, k: u32| v + ((x * 7 + y * 3 + k) % 5) as u8 - 2;
        rgba(n(c.r, 0), n(c.g, 1), n(c.b, 2))
    });
    let d = detect(&img, 3, &[]);
    assert_eq!(d.palette.colours().len(), 3);
    for x0 in [0, 10, 20] {
        let first = label(&d, x0, 0);
        for (x, y) in (x0..x0 + 10).flat_map(|x| (0..30).map(move |y| (x, y))) {
            assert_eq!(label(&d, x, y), first, "column band at x={x0} split");
        }
    }
    assert!(d.palette.colours().iter().all(|c| c.pixels == 300));
}
