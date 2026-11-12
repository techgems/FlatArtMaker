//! Colour types.

use crate::units::float_bits;
use crate::{DeltaE, Error, Result};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// An sRGB colour with no alpha, used for display colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// An sRGB pixel with straight (not premultiplied) alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// A colour in OKLab. Always finite, so it can be compared and hashed exactly.
///
/// A detected colour's OKLab value is its stable ID (SPEC "Stable IDs").
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "OklabRepr", into = "OklabRepr")]
pub struct Oklab {
    l: f64,
    a: f64,
    b: f64,
}

impl Oklab {
    /// Fails when a component is NaN or infinite.
    pub fn new(l: f64, a: f64, b: f64) -> Result<Self> {
        if l.is_finite() && a.is_finite() && b.is_finite() {
            // `+ 0.0` turns -0.0 into 0.0, so equal colours have equal bits.
            Ok(Self {
                l: l + 0.0,
                a: a + 0.0,
                b: b + 0.0,
            })
        } else {
            Err(Error::InvalidValue {
                what: "OKLab colour",
                reason: format!("components must be finite, got ({l}, {a}, {b})"),
            })
        }
    }

    pub fn l(self) -> f64 {
        self.l
    }

    pub fn a(self) -> f64 {
        self.a
    }

    pub fn b(self) -> f64 {
        self.b
    }

    /// The colour difference used by every ΔE setting.
    pub fn delta_e(self, other: Self) -> DeltaE {
        let (dl, da, db) = (self.l - other.l, self.a - other.a, self.b - other.b);
        DeltaE(100.0 * (dl * dl + da * da + db * db).sqrt())
    }
}

impl Eq for Oklab {}

impl Hash for Oklab {
    fn hash<H: Hasher>(&self, state: &mut H) {
        float_bits(self.l).hash(state);
        float_bits(self.a).hash(state);
        float_bits(self.b).hash(state);
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "tsify", tsify(rename = "Oklab"))]
struct OklabRepr {
    l: f64,
    a: f64,
    b: f64,
}

impl TryFrom<OklabRepr> for Oklab {
    type Error = Error;
    fn try_from(repr: OklabRepr) -> Result<Self> {
        Self::new(repr.l, repr.a, repr.b)
    }
}

impl From<Oklab> for OklabRepr {
    fn from(colour: Oklab) -> Self {
        Self {
            l: colour.l,
            a: colour.a,
            b: colour.b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_finite_oklab() {
        assert!(Oklab::new(f64::NAN, 0.0, 0.0).is_err());
        assert!(Oklab::new(0.5, f64::INFINITY, 0.0).is_err());
    }

    #[test]
    fn negative_zero_equals_zero() {
        let a = Oklab::new(0.5, -0.0, 0.0).unwrap();
        let b = Oklab::new(0.5, 0.0, 0.0).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.a().to_bits(), b.a().to_bits());
    }

    #[test]
    fn delta_e_is_scaled_euclidean_distance() {
        let a = Oklab::new(0.5, 0.0, 0.0).unwrap();
        let b = Oklab::new(0.5, 0.03, 0.04).unwrap();
        assert!((a.delta_e(b).0 - 5.0).abs() < 1e-9);
    }

    #[test]
    fn deserialising_rejects_non_finite() {
        let bad = r#"{"l":1e999,"a":0,"b":0}"#;
        assert!(serde_json::from_str::<Oklab>(bad).is_err());
    }
}
