//! Interfaces for the in-house stages.
//!
//! **Provisional:** these let the pipeline call in-house code without
//! depending on it. Their signatures and contracts are finished, with
//! conformance suites, once the implementations exist: `RasterStages` after
//! task 07, `ColourMapper` after task 06 and `MeshBuilder` after task 10.

use crate::Triangulator;
use flatart_domain::settings::{
    CleanUpSettings, LoadSettings, MappingSettings, MeshSettings, PrintabilitySettings,
};
use flatart_domain::{
    DetectedPalette, EditLog, LabelMap, LoadedImage, MappedColours, MeshModel, Paint, Printable,
    Result, RgbaImage, VectorModel,
};

/// The pixel stages: load (1), clean up (4), region painting (5) and printability (6).
pub trait RasterStages {
    /// Marks outside pixels (alpha 0) and finds the art bounds.
    fn load(&self, image: &RgbaImage, settings: &LoadSettings) -> Result<LoadedImage>;
    /// Mode filter, then speckle merging.
    fn clean_up(&self, labels: &LabelMap, settings: &CleanUpSettings) -> Result<LabelMap>;
    /// Fills the area under each paint point with its colour.
    fn paint(&self, labels: &LabelMap, paints: &[Paint]) -> Result<LabelMap>;
    /// Upscale and the thin-feature fix.
    fn printability(&self, labels: &LabelMap, settings: &PrintabilitySettings)
    -> Result<Printable>;
}

/// The automatic mapping plus the edit log, from detected to output colours (step 3).
pub trait ColourMapper {
    fn map(
        &self,
        detected: &DetectedPalette,
        labels: &LabelMap,
        edits: &EditLog,
        settings: &MappingSettings,
    ) -> Result<MappedColours>;
}

/// Builds the watertight mesh from the vector model (step 8).
pub trait MeshBuilder {
    fn build(
        &self,
        model: &VectorModel,
        settings: &MeshSettings,
        triangulator: &dyn Triangulator,
    ) -> Result<MeshModel>;
}
