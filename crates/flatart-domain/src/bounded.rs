//! Range-checked value types. Their constructors reject invalid values, so an
//! instance always holds a valid one. Deserialising goes through the same check.

use crate::{Error, Result};

/// Values this close outside a bound are accepted and clamped, so a UI step
/// like `0.25 + 19 * 0.05` (not exactly 1.2) isn't rejected.
const SLACK: f64 = 1e-9;

pub(crate) fn check_f64(
    what: &'static str,
    value: f64,
    min: f64,
    min_inclusive: bool,
    max: f64,
) -> Result<f64> {
    let above_min = if min_inclusive {
        value >= min - SLACK
    } else {
        value > min
    };
    if value.is_finite() && above_min && value <= max + SLACK {
        return Ok(value.clamp(min, max));
    }
    let low = if min_inclusive { "at least" } else { "above" };
    let reason = if max == f64::MAX {
        format!("must be {low} {min}, got {value}")
    } else {
        format!("must be {low} {min} and at most {max}, got {value}")
    };
    Err(Error::InvalidValue { what, reason })
}

pub(crate) fn check_int(what: &'static str, value: u64, min: u64, max: u64) -> Result<()> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(Error::InvalidValue {
            what,
            reason: format!("must be {min} to {max}, got {value}"),
        })
    }
}

/// Defines a range-checked float newtype over a unit type.
macro_rules! bounded_f64 {
    (
        $(#[$meta:meta])*
        $name:ident($unit:ident), $what:literal, min = $min:expr, min_inclusive = $min_inclusive:literal, max = $max:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
        #[serde(try_from = "f64", into = "f64")]
        #[cfg_attr(feature = "tsify", derive(tsify::Tsify), tsify(type = "number"))]
        pub struct $name(f64);

        impl $name {
            pub const MIN: f64 = $min;
            pub const MAX: f64 = $max;

            /// Fails when `value` is out of range or not finite.
            pub fn new(value: $unit) -> $crate::Result<Self> {
                $crate::bounded::check_f64($what, value.0, $min, $min_inclusive, $max).map(Self)
            }

            /// For constants known to be valid (defaults). Not every type has one.
            #[allow(dead_code)]
            pub(crate) const fn new_unchecked(value: f64) -> Self {
                Self(value)
            }

            pub fn get(self) -> $unit {
                $unit(self.0)
            }
        }

        impl TryFrom<f64> for $name {
            type Error = $crate::Error;
            fn try_from(value: f64) -> $crate::Result<Self> {
                Self::new($unit(value))
            }
        }

        impl From<$name> for f64 {
            fn from(value: $name) -> f64 {
                value.0
            }
        }

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                $crate::units::float_bits(self.0).hash(state);
            }
        }
    };
}

/// Defines a range-checked integer newtype (inclusive bounds). `as "u8"` names the
/// integer type again as a string, which `serde(try_from)` needs.
macro_rules! bounded_int {
    (
        $(#[$meta:meta])*
        $name:ident($int:ty as $int_name:literal), $what:literal, min = $min:expr, max = $max:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(try_from = $int_name, into = $int_name)]
        #[cfg_attr(feature = "tsify", derive(tsify::Tsify), tsify(type = "number"))]
        pub struct $name($int);

        impl $name {
            pub const MIN: $int = $min;
            pub const MAX: $int = $max;

            /// Fails when `value` is out of range.
            pub fn new(value: $int) -> $crate::Result<Self> {
                $crate::bounded::check_int($what, u64::from(value), u64::from(Self::MIN), u64::from(Self::MAX))
                    .map(|()| Self(value))
            }

            /// For constants known to be valid (defaults).
            pub(crate) const fn new_unchecked(value: $int) -> Self {
                Self(value)
            }

            pub fn get(self) -> $int {
                self.0
            }
        }

        impl TryFrom<$int> for $name {
            type Error = $crate::Error;
            fn try_from(value: $int) -> $crate::Result<Self> {
                Self::new(value)
            }
        }

        impl From<$name> for $int {
            fn from(value: $name) -> $int {
                value.0
            }
        }
    };
}

pub(crate) use {bounded_f64, bounded_int};
