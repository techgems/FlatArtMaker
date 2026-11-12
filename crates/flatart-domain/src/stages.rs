//! Results of the in-house stages.

use crate::{
    ColourMapping, DetectedPalette, LabelMap, OutputColourId, OutputPalette, Paint, PxPoint,
    RgbaImage, SimilarityTree,
};
use serde::Serialize;

/// The bounding box of the non-outside pixels, in source pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
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

/// Stage 3's result: the automatic mapping with the edit log applied.
#[derive(Debug, Clone, PartialEq)]
pub struct MappedColours {
    /// Every output colour, in slot order, including ones nothing uses yet.
    pub palette: OutputPalette,
    /// Which output colour each detected colour maps to.
    pub mapping: ColourMapping,
    /// The order the automatic mapping merges detected colours in, so the
    /// editor can show which tones are alike. Edits don't change it.
    pub similarity: SimilarityTree,
    /// The output label map: each cell is an `OutputColourId`, or outside.
    pub labels: LabelMap,
    /// The active paint operations, applied after clean-up (stage 5).
    pub paints: Vec<Paint>,
    /// The active operations that couldn't be applied, in log order.
    pub dropped_edits: Vec<DroppedEdit>,
}

/// An edit-log operation that was skipped, so the user can be told.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct DroppedEdit {
    /// The operation's position in the edit log.
    pub position: usize,
    pub reason: DropReason,
}

/// Why an operation was skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub enum DropReason {
    /// No detected colour is close enough to the one it names, usually after
    /// a setting changed detection.
    NoMatchingDetectedColour,
    /// It names an output colour that doesn't exist (any more).
    UnknownOutputColour,
    /// Adding would go over `MAX_OUTPUT_COLOURS` output colours.
    TooManyOutputColours,
    /// An added colour's ID is taken or below `OutputColourId::FIRST_ADDED`.
    OutputIdUnavailable,
    /// Another output colour already has that name (ignoring case).
    NameTaken,
    /// A delete that reassigns a colour to itself.
    ReassignedToItself,
    /// A reorder that lists a colour twice.
    RepeatedInOrder,
    /// A paint point outside the image.
    OutsideImage,
}

/// A part thinner than the threshold that the fix had to keep, because no
/// other colour borders it (removing it would change the outline).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct ThinFeature {
    pub at: PxPoint,
    pub colour: OutputColourId,
}

/// Stage 6's result.
#[derive(Debug, Clone, PartialEq)]
pub struct Printable {
    /// Upscaled to the trace upscale, with the thin-feature fix applied.
    pub labels: LabelMap,
    /// One per too-thin area, at its first cell in source pixels. Empty when
    /// the fix is off.
    pub thin_features: Vec<ThinFeature>,
}
