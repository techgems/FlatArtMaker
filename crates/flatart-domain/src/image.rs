//! Raster types: the input image and per-pixel label maps.

use crate::settings::TraceUpscale;
use crate::{Error, OutputColourId, Result, Rgba};
use serde::{Deserialize, Serialize};

/// Checks `width × height == len` and that neither side is zero.
fn check_grid(what: &'static str, width: u32, height: u32, len: usize) -> Result<()> {
    let expected = usize::try_from(u64::from(width) * u64::from(height)).ok();
    if width == 0 || height == 0 || expected != Some(len) {
        return Err(Error::InvalidValue {
            what,
            reason: format!("{width} × {height} doesn't match {len} cells"),
        });
    }
    Ok(())
}

fn index(width: u32, height: u32, x: u32, y: u32) -> Option<usize> {
    (x < width && y < height).then(|| y as usize * width as usize + x as usize)
}

/// An RGBA image, row by row from the top-left corner.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<Rgba>,
}

impl RgbaImage {
    /// Fails when the size is zero or doesn't match the pixel count.
    pub fn new(width: u32, height: u32, pixels: Vec<Rgba>) -> Result<Self> {
        check_grid("image", width, height, pixels.len())?;
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[Rgba] {
        &self.pixels
    }

    pub fn get(&self, x: u32, y: u32) -> Option<Rgba> {
        index(self.width, self.height, x, y).map(|i| self.pixels[i])
    }
}

/// What a label-map cell belongs to.
///
/// In a detected label map, label `n` is entry `n` of the `DetectedPalette`.
/// In an output label map, the label is the `OutputColourId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(transparent)]
pub struct Label(pub u16);

impl Label {
    /// Outside the piece (transparent or background). Never a palette entry.
    pub const OUTSIDE: Self = Self(u16::MAX);

    pub fn is_outside(self) -> bool {
        self == Self::OUTSIDE
    }

    /// The output colour, for labels in an output label map.
    pub fn output_colour(self) -> Option<OutputColourId> {
        (!self.is_outside()).then_some(OutputColourId(self.0))
    }
}

impl From<OutputColourId> for Label {
    fn from(id: OutputColourId) -> Self {
        Self(id.0)
    }
}

/// One label per cell, row by row from the top-left corner.
///
/// `upscale` cells span one source pixel in each direction (Advanced "Trace upscale").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelMap {
    width: u32,
    height: u32,
    upscale: TraceUpscale,
    labels: Vec<Label>,
}

impl LabelMap {
    /// Fails when the size is zero or doesn't match the label count.
    pub fn new(width: u32, height: u32, upscale: TraceUpscale, labels: Vec<Label>) -> Result<Self> {
        check_grid("label map", width, height, labels.len())?;
        Ok(Self {
            width,
            height,
            upscale,
            labels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn upscale(&self) -> TraceUpscale {
        self.upscale
    }

    pub fn labels(&self) -> &[Label] {
        &self.labels
    }

    pub fn get(&self, x: u32, y: u32) -> Option<Label> {
        index(self.width, self.height, x, y).map(|i| self.labels[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEAR: Rgba = Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    #[test]
    fn image_size_must_match_pixels() {
        assert!(RgbaImage::new(2, 2, vec![CLEAR; 4]).is_ok());
        assert!(RgbaImage::new(2, 2, vec![CLEAR; 3]).is_err());
        assert!(RgbaImage::new(0, 0, vec![]).is_err());
    }

    #[test]
    fn get_is_row_major_and_bounds_checked() {
        let one = TraceUpscale::new(1).unwrap();
        let map = LabelMap::new(2, 1, one, vec![Label(0), Label(1)]).unwrap();
        assert_eq!(map.get(1, 0), Some(Label(1)));
        assert_eq!(map.get(2, 0), None);
    }

    #[test]
    fn outside_has_no_output_colour() {
        assert_eq!(Label::OUTSIDE.output_colour(), None);
        assert_eq!(Label(3).output_colour(), Some(OutputColourId(3)));
    }
}
