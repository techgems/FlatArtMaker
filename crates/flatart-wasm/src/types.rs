//! Results the engine hands to the browser that aren't domain types. Their
//! TypeScript declarations are generated (tsify) alongside the domain's.
//!
//! Large buffers (label maps, mesh vertices) aren't here: they cross as typed
//! arrays from their own calls, so the worker can transfer them.

use flatart_domain::{
    ArtBounds, ColourMapping, DetectedPalette, DroppedEdit, OutputColourId, OutputPalette,
    PartKind, Rgb, Settings, SimilarityTree, ThinFeature,
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;

/// What loading an image gives back.
#[derive(Debug, Clone, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LoadedImageInfo {
    /// The art's bounding box in source pixels; the print size follows it.
    pub bounds: ArtBounds,
    /// The SPEC defaults, sized to 100 mm on the art's longest side.
    pub settings: Settings,
}

/// The colour-mapping stage's result, without its label map.
#[derive(Debug, Clone, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct MappedInfo {
    /// The label map's size in cells (one cell per source pixel).
    pub width: u32,
    pub height: u32,
    /// The label for cells outside the piece. Every other label is an
    /// `OutputColourId`.
    pub outside: u16,
    pub palette: OutputPalette,
    pub detected: DetectedPalette,
    pub mapping: ColourMapping,
    pub similarity: SimilarityTree,
}

/// What the user should be told about the current result.
#[derive(Debug, Clone, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Too-thin areas the thin-feature fix had to keep.
    pub thin_features: Vec<ThinFeature>,
    /// Edit-log operations that couldn't be applied.
    pub dropped_edits: Vec<DroppedEdit>,
}

/// One printable part, without its mesh buffers.
#[derive(Debug, Clone, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct MeshPartInfo {
    pub kind: PartKind,
    pub colour: OutputColourId,
    /// `Base` for the base, otherwise the colour's name.
    pub name: String,
    pub display: Rgb,
    /// The filament slot (and extruder). `undefined` only if the part's
    /// colour isn't in the palette.
    pub slot: Option<u8>,
}

/// Which slicer a 3MF is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum Slicer {
    /// PrusaSlicer.
    Prusa,
    /// Bambu Studio and Orca-based slicers.
    Bambu,
}

impl Slicer {
    /// The pipeline's exporter id.
    pub fn exporter(self) -> &'static str {
        match self {
            Slicer::Prusa => "3mf-prusa",
            Slicer::Bambu => "3mf-bambu",
        }
    }
}
