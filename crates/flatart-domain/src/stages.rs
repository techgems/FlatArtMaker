//! Results of the in-house stages.
//!
//! **Provisional:** these take their final shape once `flatart-raster`
//! (task 07), the colour mapping (task 06) and the mesh builder (task 10) exist.

use crate::{
    ColourMapping, DetectedPalette, LabelMap, OutputColourId, OutputPalette, Paint, PxPoint,
    RgbaImage,
};

/// The bounding box of the non-outside pixels, in source pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArtBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Stage 1's result. Alpha is 0 for outside pixels and 255 for all others.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedImage {
    pub image: RgbaImage,
    pub bounds: ArtBounds,
}

/// Stage 2's result.
#[derive(Debug, Clone, PartialEq)]
pub struct Detection {
    pub palette: DetectedPalette,
    /// Labels index `palette`.
    pub labels: LabelMap,
}

/// Stage 3's result: the edit log applied to the detected colours.
#[derive(Debug, Clone, PartialEq)]
pub struct MappedColours {
    pub palette: OutputPalette,
    pub mapping: ColourMapping,
    /// The output label map.
    pub labels: LabelMap,
    /// The active paint operations, applied after clean-up.
    pub paints: Vec<Paint>,
    /// Positions in the edit log of operations that no longer match a detected colour.
    pub dropped_edits: Vec<usize>,
}

/// A feature thinner than the threshold that the fix couldn't remove.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThinFeature {
    pub at: PxPoint,
    pub colour: OutputColourId,
}

/// Stage 6's result.
#[derive(Debug, Clone, PartialEq)]
pub struct Printable {
    pub labels: LabelMap,
    pub thin_features: Vec<ThinFeature>,
}
