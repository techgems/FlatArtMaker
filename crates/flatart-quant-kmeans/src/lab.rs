//! The working colour space: OKLab as plain `f64`s, converted with `palette`.

use flatart_domain::{Oklab, Result};
use palette::{IntoColor, Srgb};

/// An OKLab colour inside this crate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Lab(pub [f64; 3]);

impl Lab {
    pub(crate) fn from_srgb(rgb: [u8; 3]) -> Self {
        let linear = Srgb::new(rgb[0], rgb[1], rgb[2])
            .into_format::<f64>()
            .into_linear();
        let lab: palette::Oklab<f64> = linear.into_color();
        Self([lab.l, lab.a, lab.b])
    }

    pub(crate) fn from_domain(colour: Oklab) -> Self {
        Self([colour.l(), colour.a(), colour.b()])
    }

    pub(crate) fn to_domain(self) -> Result<Oklab> {
        Oklab::new(self.0[0], self.0[1], self.0[2])
    }

    /// Squared Euclidean distance.
    pub(crate) fn distance2(self, other: Self) -> f64 {
        let [dl, da, db] = [0, 1, 2].map(|i| self.0[i] - other.0[i]);
        dl * dl + da * da + db * db
    }
}

/// The index of the centre nearest `point`; ties go to the lower index.
pub(crate) fn nearest(point: Lab, centres: &[Lab]) -> usize {
    let mut best = (0, f64::INFINITY);
    for (i, &centre) in centres.iter().enumerate() {
        let d = point.distance2(centre);
        if d < best.1 {
            best = (i, d);
        }
    }
    best.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(lab: Lab, expected: [f64; 3]) -> bool {
        lab.0
            .iter()
            .zip(expected)
            .all(|(a, b)| (a - b).abs() < 1e-4)
    }

    #[test]
    fn converts_reference_colours() {
        assert!(close(Lab::from_srgb([255, 255, 255]), [1.0, 0.0, 0.0]));
        assert!(close(Lab::from_srgb([0, 0, 0]), [0.0, 0.0, 0.0]));
        // Björn Ottosson's reference value for sRGB red.
        assert!(close(
            Lab::from_srgb([255, 0, 0]),
            [0.627_955, 0.224_863, 0.125_846]
        ));
    }

    #[test]
    fn nearest_prefers_the_lower_index_on_ties() {
        let centres = [Lab([0.0, 0.1, 0.0]), Lab([0.0, -0.1, 0.0])];
        assert_eq!(nearest(Lab([0.0; 3]), &centres), 0);
        assert_eq!(nearest(Lab([0.0, -0.09, 0.0]), &centres), 1);
    }
}
