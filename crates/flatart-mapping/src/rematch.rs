//! Finds the detected colour an edit-log operation refers to (SPEC "Stable IDs").

use crate::colour_space::{delta_e, lab};
use flatart_domain::{DeltaE, DetectedPalette, Oklab};

/// How far a stored detected colour may be from a current one and still
/// match. Large enough to survive the shift from changing the detected-colour
/// count, small enough not to grab a different colour.
pub const REMATCH_LIMIT: DeltaE = DeltaE(10.0);

/// The index of the detected colour nearest `stored`, if within
/// [`REMATCH_LIMIT`]. Ties go to the lower index.
pub(crate) fn find(detected: &DetectedPalette, stored: Oklab) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (i, c) in detected.colours().iter().enumerate() {
        let d = delta_e(lab(c.colour), lab(stored));
        if d <= REMATCH_LIMIT.0 && best.is_none_or(|(_, b)| d < b) {
            best = Some((i, d));
        }
    }
    best.map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatart_domain::DetectedColour;

    fn palette(ls: &[f64]) -> DetectedPalette {
        let colours = ls
            .iter()
            .map(|&l| DetectedColour {
                colour: Oklab::new(l, 0.0, 0.0).unwrap(),
                pixels: 1,
            })
            .collect();
        DetectedPalette::new(colours).unwrap()
    }

    #[test]
    fn matches_exactly_nearest_or_not_at_all() {
        let p = palette(&[0.2, 0.5, 0.8]);
        let at = |l| find(&p, Oklab::new(l, 0.0, 0.0).unwrap());
        assert_eq!(at(0.5), Some(1));
        assert_eq!(at(0.56), Some(1));
        assert_eq!(at(0.66), None, "16 ΔE from both neighbours");
        assert_eq!(at(0.35), None);
    }
}
