//! sRGB ↔ OKLab, after Björn Ottosson's reference implementation.

use flatart_domain::{Oklab, Rgb};

/// OKLab as plain numbers: `[L, a, b]`.
pub(crate) type Lab = [f64; 3];

pub(crate) fn lab(colour: Oklab) -> Lab {
    [colour.l(), colour.a(), colour.b()]
}

/// The ΔE used by every setting: 100 × Euclidean distance in OKLab.
pub(crate) fn delta_e(p: Lab, q: Lab) -> f64 {
    let d: f64 = p.iter().zip(q).map(|(a, b)| (a - b) * (a - b)).sum();
    100.0 * d.sqrt()
}

pub(crate) fn from_rgb(rgb: Rgb) -> Lab {
    let [r, g, b] = [rgb.r, rgb.g, rgb.b].map(|c| to_linear(f64::from(c) / 255.0));
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

/// The nearest displayable sRGB colour; out-of-gamut channels are clamped.
pub(crate) fn to_rgb([lightness, a, b]: Lab) -> Rgb {
    let l = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
    let linear = [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
    ];
    let [r, g, b] = linear.map(|c| (to_gamma(c.clamp(0.0, 1.0)) * 255.0).round() as u8);
    Rgb { r, g, b }
}

fn to_linear(c: f64) -> f64 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn to_gamma(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_reference_values() {
        let red = from_rgb(Rgb { r: 255, g: 0, b: 0 });
        let expected = [0.627_955, 0.224_863, 0.125_846];
        assert!(red.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-5));
        let white = from_rgb(Rgb {
            r: 255,
            g: 255,
            b: 255,
        });
        assert!((white[0] - 1.0).abs() < 1e-6 && white[1].abs() < 1e-6);
    }

    #[test]
    fn round_trips_every_grey_and_some_colours() {
        let mut samples: Vec<Rgb> = (0..=255).map(|v| Rgb { r: v, g: v, b: v }).collect();
        samples.extend([
            Rgb { r: 255, g: 0, b: 0 },
            Rgb {
                r: 12,
                g: 200,
                b: 99,
            },
            Rgb { r: 1, g: 2, b: 250 },
        ]);
        for rgb in samples {
            assert_eq!(to_rgb(from_rgb(rgb)), rgb);
        }
    }

    #[test]
    fn clamps_out_of_gamut() {
        assert_eq!(
            to_rgb([1.5, 0.0, 0.0]),
            Rgb {
                r: 255,
                g: 255,
                b: 255
            }
        );
        assert_eq!(to_rgb([-0.5, 0.0, 0.0]), Rgb { r: 0, g: 0, b: 0 });
    }
}
