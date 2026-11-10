//! Automatic output-colour names: the nearest common colour name, made unique.

use crate::colour_space::{delta_e, from_rgb};
use flatart_domain::{ColourName, Result, Rgb};

/// Common colour names, chosen to read like filament colours. Black and
/// DarkGrey sit a little lighter than their textbook values, so a very dark
/// grey like (30, 30, 30) still reads as black.
const NAMES: &[(&str, [u8; 3])] = &[
    ("Black", [20, 20, 20]),
    ("DarkGrey", [80, 80, 80]),
    ("Grey", [128, 128, 128]),
    ("LightGrey", [192, 192, 192]),
    ("White", [255, 255, 255]),
    ("Cream", [255, 250, 225]),
    ("Beige", [230, 215, 180]),
    ("Tan", [210, 180, 140]),
    ("Brown", [130, 75, 30]),
    ("DarkBrown", [80, 45, 20]),
    ("DarkRed", [139, 0, 0]),
    ("Red", [230, 25, 25]),
    ("Salmon", [250, 128, 114]),
    ("Coral", [255, 110, 80]),
    ("Orange", [255, 140, 0]),
    ("Peach", [255, 200, 160]),
    ("OrangeYellow", [255, 190, 0]),
    ("Gold", [212, 175, 55]),
    ("Yellow", [255, 230, 0]),
    ("LightYellow", [255, 250, 160]),
    ("Olive", [128, 128, 0]),
    ("Lime", [150, 230, 30]),
    ("LightGreen", [144, 238, 144]),
    ("Green", [30, 160, 50]),
    ("DarkGreen", [0, 100, 0]),
    ("Mint", [170, 240, 200]),
    ("Teal", [0, 128, 128]),
    ("Turquoise", [64, 224, 208]),
    ("Cyan", [0, 200, 230]),
    ("SkyBlue", [135, 206, 235]),
    ("LightBlue", [173, 216, 230]),
    ("Blue", [30, 80, 220]),
    ("Navy", [0, 0, 128]),
    ("Indigo", [75, 0, 130]),
    ("Purple", [128, 40, 160]),
    ("Violet", [180, 120, 230]),
    ("Lavender", [210, 190, 240]),
    ("Magenta", [220, 0, 200]),
    ("HotPink", [255, 105, 180]),
    ("Pink", [255, 150, 190]),
    ("LightPink", [255, 200, 215]),
];

/// How much more a chroma (a, b) difference counts than a lightness one when
/// naming. Plain OKLab distance calls a near-black like (22, 22, 22) "dark
/// brown"; for names, hue matters more than a small step in lightness.
const CHROMA_WEIGHT: f64 = 3.0;

/// The common name nearest `colour` in OKLab, with chroma weighted up.
fn nearest(colour: Rgb) -> &'static str {
    let [l, a, b] = from_rgb(colour);
    let target = [l, a * CHROMA_WEIGHT, b * CHROMA_WEIGHT];
    let distance = |rgb: [u8; 3]| {
        let [r, g, b] = rgb;
        let [l, a, b] = from_rgb(Rgb { r, g, b });
        delta_e(target, [l, a * CHROMA_WEIGHT, b * CHROMA_WEIGHT])
    };
    NAMES
        .iter()
        .min_by(|x, y| distance(x.1).total_cmp(&distance(y.1)))
        .map_or("Colour", |(name, _)| name)
}

/// The nearest common name, with `2`, `3`, … appended when `taken` already
/// has it (ignoring case).
pub(crate) fn auto_name(colour: Rgb, taken: &[ColourName]) -> Result<ColourName> {
    let base = nearest(colour);
    let free = |name: &str| !taken.iter().any(|t| t.as_str().eq_ignore_ascii_case(name));
    // At most `taken.len()` candidates can be taken, so one of these is free.
    let name = (1..=taken.len() + 1)
        .map(|n| match n {
            1 => base.to_owned(),
            n => format!("{base}{n}"),
        })
        .find(|name| free(name))
        .unwrap_or_else(|| base.to_owned());
    ColourName::new(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_table_name_is_valid_and_unique() {
        for (i, (name, _)) in NAMES.iter().enumerate() {
            assert!(ColourName::new(*name).is_ok(), "{name}");
            assert!(
                NAMES[i + 1..]
                    .iter()
                    .all(|(other, _)| !other.eq_ignore_ascii_case(name)),
                "{name} repeats"
            );
        }
    }

    #[test]
    fn picks_the_nearest_name() {
        let rgb = |r, g, b| Rgb { r, g, b };
        assert_eq!(nearest(rgb(10, 10, 12)), "Black");
        assert_eq!(nearest(rgb(30, 30, 30)), "Black");
        assert_eq!(nearest(rgb(237, 237, 237)), "White");
        assert_eq!(nearest(rgb(250, 228, 20)), "Yellow");
        assert_eq!(nearest(rgb(250, 185, 10)), "OrangeYellow");
        assert_eq!(nearest(rgb(20, 60, 200)), "Blue");
        assert_eq!(nearest(rgb(110, 65, 30)), "Brown");
    }

    #[test]
    fn appends_a_number_when_taken() {
        let black = Rgb { r: 0, g: 0, b: 0 };
        let taken = vec![
            ColourName::new("black").unwrap(),
            ColourName::new("Black2").unwrap(),
        ];
        assert_eq!(auto_name(black, &taken).unwrap().as_str(), "Black3");
        assert_eq!(auto_name(black, &[]).unwrap().as_str(), "Black");
    }
}
