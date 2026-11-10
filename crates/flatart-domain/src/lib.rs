//! Data types shared by every FlatArtMaker crate.
//!
//! Interface signatures in `flatart-ports` use only these types, so no
//! third-party type ever crosses a port boundary. Constructors reject invalid
//! values, so a value of these types is always valid.

mod border_map;
mod bounded;
mod colour;
mod edit;
mod engine_info;
mod error;
mod geometry;
mod image;
mod models;
mod palette;
pub mod settings;
mod similarity;
mod stages;
mod units;

pub use border_map::{
    Border, BorderEnds, BorderId, BorderMap, Face, Junction, JunctionId, Loop, LoopStep,
};
pub use colour::{Oklab, Rgb, Rgba};
pub use edit::{EditLog, EditOp, Paint};
pub use engine_info::EngineInfo;
pub use error::{Error, Result, Stage};
pub use geometry::{CubicBezier, MmPoint, MmPoint3, PolygonWithHoles, PxPoint};
pub use image::{Label, LabelMap, RgbaImage};
pub use models::{ExportOptions, Mesh, MeshModel, MeshPart, PackageFile, PartKind, VectorModel};
pub use palette::{
    BASE_NAME, ColourMapping, ColourName, DetectedColour, DetectedPalette, MAX_OUTPUT_COLOURS,
    OutputColour, OutputColourId, OutputPalette, Slot,
};
pub use settings::Settings;
pub use similarity::{GroupId, Merge, SimilarityTree};
pub use stages::{
    ArtBounds, Detection, DropReason, DroppedEdit, LoadedImage, MappedColours, Printable,
    ThinFeature,
};
pub use units::{Degrees, DeltaE, Mm, MmPerPx, Px};
