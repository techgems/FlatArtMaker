//! Detected colours, output colours (filament slots) and the mapping between them.

use crate::{Error, Label, Oklab, Result, Rgb};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The most output colours a piece can have. The user picks how many they
/// want (`OutputColourTarget`); this is the ceiling on that choice.
pub const MAX_OUTPUT_COLOURS: usize = 16;

/// The name the base part always has, so no output colour may use it.
pub const BASE_NAME: &str = "Base";

/// One colour found by detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct DetectedColour {
    /// Also the colour's stable ID.
    pub colour: Oklab,
    /// How many source pixels have this colour as their nearest.
    pub pixels: u64,
}

/// The detected colours. Entry `n` is label `n` of the detected label map.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct DetectedPalette {
    colours: Vec<DetectedColour>,
}

impl DetectedPalette {
    /// Fails when there are too many colours for a label to address.
    pub fn new(colours: Vec<DetectedColour>) -> Result<Self> {
        if colours.len() >= usize::from(Label::OUTSIDE.0) {
            return Err(Error::InvalidValue {
                what: "detected palette",
                reason: format!("{} colours is too many", colours.len()),
            });
        }
        Ok(Self { colours })
    }

    pub fn colours(&self) -> &[DetectedColour] {
        &self.colours
    }

    /// Share of all opaque pixels, from 0 to 1.
    pub fn coverage(&self, index: usize) -> f64 {
        let total: u64 = self.colours.iter().map(|c| c.pixels).sum();
        match self.colours.get(index) {
            Some(c) if total > 0 => c.pixels as f64 / total as f64,
            _ => 0.0,
        }
    }
}

/// A generated ID that never changes, even when the colour or name does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(transparent)]
pub struct OutputColourId(pub u16);

impl OutputColourId {
    /// IDs below this belong to the automatic mapping's colours (slot order
    /// at the time). Colours the user adds take this ID or higher, so they
    /// never collide when a setting changes how many automatic colours exist.
    pub const FIRST_ADDED: Self = Self(MAX_OUTPUT_COLOURS as u16);
}

/// An output colour's name: 1–32 ASCII letters and digits, never `Base`.
///
/// It names the 3MF part, the STEP component and bodies, and the SVG layer ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "tsify", tsify(type = "string"))]
#[serde(try_from = "String", into = "String")]
pub struct ColourName(String);

impl ColourName {
    pub const MAX_LEN: usize = 32;

    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let reason = if name.is_empty() || name.len() > Self::MAX_LEN {
            Some(format!("must be 1 to {} characters", Self::MAX_LEN))
        } else if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
            Some("may only use letters and digits".to_owned())
        } else if name.eq_ignore_ascii_case(BASE_NAME) {
            Some(format!("`{BASE_NAME}` is reserved for the base"))
        } else {
            None
        };
        match reason {
            Some(reason) => Err(Error::InvalidValue {
                what: "colour name",
                reason: format!("{reason}, got `{name}`"),
            }),
            None => Ok(Self(name)),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ColourName {
    type Error = Error;
    fn try_from(name: String) -> Result<Self> {
        Self::new(name)
    }
}

impl From<ColourName> for String {
    fn from(name: ColourName) -> String {
        name.0
    }
}

/// A filament slot, starting at 1. Output colour slot *n* prints on extruder *n*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(transparent)]
pub struct Slot(u8);

impl Slot {
    pub fn get(self) -> u8 {
        self.0
    }
}

/// One output colour: a filament.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct OutputColour {
    pub id: OutputColourId,
    pub name: ColourName,
    /// The colour shown and exported.
    pub display: Rgb,
}

/// The output colours in slot order: the first is slot 1.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "OutputPaletteRepr", into = "OutputPaletteRepr")]
pub struct OutputPalette {
    colours: Vec<OutputColour>,
    base: Option<OutputColourId>,
}

impl OutputPalette {
    /// Fails on more than [`MAX_OUTPUT_COLOURS`] colours, a repeated ID or name (ignoring case),
    /// an ID equal to `Label::OUTSIDE`, or a base that isn't in the list.
    pub fn new(colours: Vec<OutputColour>, base: Option<OutputColourId>) -> Result<Self> {
        let invalid = |reason: String| Error::InvalidValue {
            what: "output palette",
            reason,
        };
        if colours.len() > MAX_OUTPUT_COLOURS {
            return Err(invalid(format!(
                "{} colours is more than {MAX_OUTPUT_COLOURS}",
                colours.len()
            )));
        }
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        for colour in &colours {
            if Label::from(colour.id).is_outside() {
                return Err(invalid(format!("ID {} is reserved", colour.id.0)));
            }
            if !ids.insert(colour.id) {
                return Err(invalid(format!("ID {} is repeated", colour.id.0)));
            }
            if !names.insert(colour.name.as_str().to_ascii_lowercase()) {
                return Err(invalid(format!(
                    "name `{}` is repeated",
                    colour.name.as_str()
                )));
            }
        }
        if let Some(base) = base
            && !ids.contains(&base)
        {
            return Err(invalid(format!("base ID {} is not in the palette", base.0)));
        }
        Ok(Self { colours, base })
    }

    pub fn colours(&self) -> &[OutputColour] {
        &self.colours
    }

    /// The base part's colour, whose faces merge into the base. Ignored when
    /// B = 0 (no base). `None` only when there are no output colours.
    pub fn base(&self) -> Option<OutputColourId> {
        self.base
    }

    pub fn get(&self, id: OutputColourId) -> Option<&OutputColour> {
        self.colours.iter().find(|c| c.id == id)
    }

    /// The filament slot of `id` among the colours `in_use`: colours that
    /// reach no part take no slot, so the slots run 1, 2, 3, … without gaps,
    /// in palette order. `None` when `id` isn't in the palette or in use.
    pub fn slot(
        &self,
        id: OutputColourId,
        in_use: impl Fn(OutputColourId) -> bool,
    ) -> Option<Slot> {
        if !in_use(id) {
            return None;
        }
        let before = self
            .colours
            .iter()
            .map(|c| c.id)
            .take_while(|&c| c != id)
            .filter(|&c| in_use(c))
            .count();
        self.get(id)?;
        // At most 16 colours, so the slot always fits.
        u8::try_from(before + 1).ok().map(Slot)
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "tsify", tsify(rename = "OutputPalette"))]
struct OutputPaletteRepr {
    colours: Vec<OutputColour>,
    base: Option<OutputColourId>,
}

impl TryFrom<OutputPaletteRepr> for OutputPalette {
    type Error = Error;
    fn try_from(repr: OutputPaletteRepr) -> Result<Self> {
        Self::new(repr.colours, repr.base)
    }
}

impl From<OutputPalette> for OutputPaletteRepr {
    fn from(palette: OutputPalette) -> Self {
        Self {
            colours: palette.colours,
            base: palette.base,
        }
    }
}

/// Which output colour each detected colour maps to. Entry `n` is for detected label `n`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct ColourMapping {
    pub targets: Vec<OutputColourId>,
}

impl ColourMapping {
    pub fn target(&self, detected: Label) -> Option<OutputColourId> {
        self.targets.get(usize::from(detected.0)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn colour(id: u16, name: &str) -> OutputColour {
        OutputColour {
            id: OutputColourId(id),
            name: ColourName::new(name).unwrap(),
            display: Rgb { r: 0, g: 0, b: 0 },
        }
    }

    #[test]
    fn names_are_letters_and_digits_and_not_base() {
        assert!(ColourName::new("OrangeYellow2").is_ok());
        assert!(ColourName::new("").is_err());
        assert!(ColourName::new("Dark Red").is_err());
        assert!(ColourName::new("Grün").is_err());
        assert!(ColourName::new("base").is_err());
        assert!(ColourName::new("x".repeat(33)).is_err());
    }

    #[test]
    fn palette_allows_at_most_sixteen_colours() {
        let make = |n: usize| {
            (0..n)
                .map(|i| colour(i as u16, &format!("C{i}")))
                .collect::<Vec<_>>()
        };
        assert!(OutputPalette::new(make(16), None).is_ok());
        assert!(OutputPalette::new(make(17), None).is_err());
    }

    #[test]
    fn palette_rejects_repeats_and_unknown_base() {
        let dup_id = vec![colour(1, "Black"), colour(1, "White")];
        assert!(OutputPalette::new(dup_id, None).is_err());
        let dup_name = vec![colour(1, "Black"), colour(2, "BLACK")];
        assert!(OutputPalette::new(dup_name, None).is_err());
        let ok = vec![colour(1, "Black"), colour(2, "White")];
        assert!(OutputPalette::new(ok.clone(), Some(OutputColourId(3))).is_err());
        assert!(OutputPalette::new(vec![colour(u16::MAX, "Black")], None).is_err());
        let palette = OutputPalette::new(ok, Some(OutputColourId(2))).unwrap();
        let all = |_| true;
        assert_eq!(palette.slot(OutputColourId(2), all).map(Slot::get), Some(2));
        assert_eq!(palette.slot(OutputColourId(9), all), None);
    }

    #[test]
    fn empty_colours_take_no_slot() {
        let palette = OutputPalette::new(
            vec![colour(1, "Black"), colour(2, "White"), colour(3, "Red")],
            None,
        )
        .unwrap();
        let in_use = |id: OutputColourId| id != OutputColourId(2);
        let slot = |id| palette.slot(OutputColourId(id), in_use).map(Slot::get);
        assert_eq!([slot(1), slot(2), slot(3)], [Some(1), None, Some(2)]);
    }

    #[test]
    fn coverage_is_a_share_of_pixels() {
        let c = Oklab::new(0.5, 0.0, 0.0).unwrap();
        let palette = DetectedPalette::new(vec![
            DetectedColour {
                colour: c,
                pixels: 1,
            },
            DetectedColour {
                colour: c,
                pixels: 3,
            },
        ])
        .unwrap();
        assert!((palette.coverage(1) - 0.75).abs() < 1e-12);
        assert_eq!(palette.coverage(5), 0.0);
    }
}
