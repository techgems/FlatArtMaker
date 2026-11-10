//! Unit newtypes. Geometry never travels as a bare `f64`.

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// The bits of `value` for hashing, with `-0.0` folded into `0.0` so equal values hash equally.
pub(crate) fn float_bits(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

macro_rules! unit {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub f64);

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                float_bits(self.0).hash(state);
            }
        }

        impl From<f64> for $name {
            fn from(value: f64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for f64 {
            fn from(value: $name) -> f64 {
                value.0
            }
        }
    };
}

unit!(
    /// A length in millimetres.
    Mm
);
unit!(
    /// A length or coordinate in source-image pixels.
    Px
);
unit!(
    /// Millimetres per source-image pixel.
    MmPerPx
);
unit!(
    /// An angle in degrees.
    Degrees
);
unit!(
    /// A colour difference: Euclidean distance in OKLab, times 100.
    DeltaE
);
