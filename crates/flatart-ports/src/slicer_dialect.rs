use flatart_domain::{MeshModel, PackageFile, Result};

/// The 3MF model part, which every dialect writes.
pub const MODEL_PATH: &str = "3D/3dmodel.model";

/// The slicer-specific part of a 3MF (Pipeline step 9).
///
/// A dialect writes the model part (its part layout and part names) and the
/// slicer's config under `Metadata/` (the extruder per part). The 3MF
/// exporter owns the rest of the package: `[Content_Types].xml`, `_rels/`,
/// the thumbnail and the zip.
///
/// # Contract
/// - **Parts:** one part per `MeshPart`, named by `MeshModel::part_name`,
///   on extruder `MeshModel::slot` (output colour slot *n* on extruder *n*).
/// - **Files:** always includes [`MODEL_PATH`]. Paths are relative, use `/`,
///   don't repeat, and never collide with the exporter's own files.
/// - **Deterministic.**
pub trait SlicerDialect {
    /// Short and stable, e.g. `prusa` or `bambu`.
    fn id(&self) -> &str;
    fn parts(&self, model: &MeshModel) -> Result<Vec<PackageFile>>;
}
